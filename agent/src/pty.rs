//! PTY session and process-tree teardown (doc 7.5).
//!
//! The teardown is the part that needs care. `portable_pty::Child::kill()` sends
//! a signal to the shell alone, so anything the user started under it - a dev
//! server, a watch task - survives and keeps holding the terminal's file
//! descriptors. Doc 7.5 therefore fixes a platform-specific mechanism:
//! a process group on Unix, a Job Object on Windows.

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

use crate::AgentError;

pub struct PtySession {
    /// `Option` so teardown can release the pty before reaping the child:
    /// on Windows, ConPTY keeps the child alive until the pseudoconsole is
    /// closed, so waiting first deadlocks (observed in the 2026-07-31
    /// Windows acceptance run: session_table tests hung, Ctrl-C never
    /// finished shutting down).
    master: Option<Box<dyn MasterPty + Send>>,
    child: Box<dyn Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
    pub shell: String,
    #[cfg(windows)]
    job: Option<windows_job::JobObject>,
}

impl PtySession {
    /// Spawns the shell. `portable-pty` already puts the child in its own
    /// session on Unix (it must, to make the pty its controlling terminal), so
    /// the child pid doubles as the process group id used at teardown.
    pub fn spawn(
        program: &str,
        args: &[String],
        cwd: Option<&std::path::Path>,
        cols: u16,
        rows: u16,
    ) -> Result<(Self, Box<dyn Read + Send>), AgentError> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AgentError::Pty(format!("cannot open a pty: {e}")))?;

        let mut cmd = CommandBuilder::new(program);
        for arg in args {
            cmd.arg(arg);
        }
        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }
        // Without this the remote shell reports a dumb terminal and colour and
        // cursor addressing stop working in xterm.js.
        cmd.env("TERM", "xterm-256color");

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| AgentError::Pty(format!("cannot start {program}: {e}")))?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| AgentError::Pty(format!("cannot read from the pty: {e}")))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| AgentError::Pty(format!("cannot write to the pty: {e}")))?;

        #[cfg(windows)]
        let job = child.process_id().and_then(windows_job::JobObject::capture);

        Ok((
            Self {
                master: Some(pair.master),
                child,
                writer,
                shell: program.to_string(),
                #[cfg(windows)]
                job,
            },
            reader,
        ))
    }

    pub fn write_input(&mut self, data: &[u8]) -> Result<(), AgentError> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), AgentError> {
        let Some(master) = &self.master else {
            return Err(AgentError::Pty("the session is already terminated".into()));
        };
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| AgentError::Pty(format!("resize failed: {e}")))
    }

    pub fn try_wait(&mut self) -> Option<i32> {
        match self.child.try_wait() {
            Ok(Some(status)) => Some(status.exit_code() as i32),
            _ => None,
        }
    }

    /// Terminates the shell and everything it started. Guaranteed to return
    /// within a few seconds: nothing here waits unboundedly, because this
    /// runs inside async teardown paths (serve.rs) and drop handlers where
    /// a hang wedges the whole process (exactly what the 2026-07-31 Windows
    /// run demonstrated).
    pub fn terminate(&mut self) {
        let pid = self.child.process_id();

        #[cfg(unix)]
        if let Some(pid) = pid {
            unix_teardown::kill_process_group(pid as i32);
        }

        #[cfg(windows)]
        {
            let _ = pid;
            if let Some(job) = self.job.take() {
                // Closing the job handle terminates every process in it, because
                // the job was created with JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE.
                drop(job);
            }
        }

        // Release the pty before reaping: ConPTY does not let the child
        // finish dying while the pseudoconsole is open, and a closed master
        // also unblocks any I/O still pending on the pipes. Readers cloned
        // from the master earlier stay valid (they hold their own handles).
        drop(self.master.take());

        let _ = self.child.kill();

        // Bounded reap instead of a blocking wait(). If the child has not
        // become reapable a few seconds after a kill, waiting longer will
        // not save it - and blocking here forever takes the agent with it.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if self.child.try_wait().ok().flatten().is_some() {
                return;
            }
            if std::time::Instant::now() >= deadline {
                tracing::warn!("a shell process did not become reapable within 5s of being killed");
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}

impl Drop for PtySession {
    fn drop(&mut self) {
        self.terminate();
    }
}

#[cfg(unix)]
mod unix_teardown {
    //! Teardown by session, not by process group.
    //!
    //! killpg on the shell's pid looks right but is not enough: once the shell
    //! enables job control - which it does as soon as it sees a tty - a
    //! backgrounded job is placed in its own process group, so `kill(-shellpid)`
    //! never reaches it. A `sleep 300 &` started from the remote terminal
    //! survives, which is exactly the orphan doc 7.5 exists to prevent.
    //!
    //! Job control moves processes between groups but never between sessions,
    //! so the session is the set that actually corresponds to "this terminal".
    //! The group signal is still sent first because it is cheap and reaches the
    //! common case without walking /proc.

    use std::time::{Duration, Instant};

    const SIGHUP: i32 = 1;
    const SIGKILL: i32 = 9;

    pub fn kill_process_group(pid: i32) {
        let sid = unsafe { getsid(pid) };

        unsafe {
            kill(-pid, SIGHUP);
        }
        if sid > 0 {
            signal_session(sid, pid, SIGHUP);
        }

        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
            if session_members(sid, pid).is_empty() && unsafe { kill(-pid, 0) } != 0 {
                return;
            }
        }

        unsafe {
            kill(-pid, SIGKILL);
        }
        if sid > 0 {
            signal_session(sid, pid, SIGKILL);
        }
    }

    fn signal_session(sid: i32, shell_pid: i32, signal: i32) {
        for pid in session_members(sid, shell_pid) {
            unsafe {
                kill(pid, signal);
            }
        }
    }

    /// Every live pid in `sid`, plus the shell itself, read from /proc.
    ///
    /// Linux-only, which matches the supported target set (doc 1.2). Field 6 of
    /// /proc/<pid>/stat is the session id; parsing starts after the last ')'
    /// because the comm field can contain spaces and parentheses.
    fn session_members(sid: i32, shell_pid: i32) -> Vec<i32> {
        if sid <= 0 {
            return Vec::new();
        }

        let self_pid = std::process::id() as i32;
        let mut found = Vec::new();

        let Ok(entries) = std::fs::read_dir("/proc") else {
            return found;
        };

        for entry in entries.flatten() {
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            let Ok(pid) = name.parse::<i32>() else {
                continue;
            };
            if pid == self_pid {
                continue;
            }

            let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
                continue;
            };
            let Some(rest) = stat.rsplit_once(')').map(|(_, r)| r) else {
                continue;
            };

            // rest starts with " S ppid pgrp session ..."
            let mut fields = rest.split_whitespace();
            let session = fields.nth(3).and_then(|v| v.parse::<i32>().ok());

            if session == Some(sid) || pid == shell_pid {
                found.push(pid);
            }
        }

        found
    }

    extern "C" {
        fn kill(pid: i32, sig: i32) -> i32;
        fn getsid(pid: i32) -> i32;
    }
}

#[cfg(windows)]
mod windows_job {
    //! Job Object teardown (doc 7.5).
    //!
    //! Untested on this build host - there is no Windows machine in the loop -
    //! so the acceptance run on Windows must confirm that a child started from
    //! the remote shell really does die with the session.

    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
    };

    pub struct JobObject(HANDLE);

    unsafe impl Send for JobObject {}
    unsafe impl Sync for JobObject {}

    impl JobObject {
        pub fn capture(pid: u32) -> Option<Self> {
            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if job.is_null() {
                    return None;
                }

                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

                if SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const core::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                ) == 0
                {
                    CloseHandle(job);
                    return None;
                }

                let process = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, 0, pid);
                if process.is_null() {
                    CloseHandle(job);
                    return None;
                }

                let assigned = AssignProcessToJobObject(job, process);
                CloseHandle(process);

                if assigned == 0 {
                    CloseHandle(job);
                    return None;
                }

                Some(Self(job))
            }
        }
    }

    impl Drop for JobObject {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

/// Doc 4.5 and 7.5: one active session per agent, a second request is refused
/// with DEVICE_BUSY rather than queued.
#[derive(Clone, Default)]
pub struct SessionSlot {
    inner: Arc<Mutex<Option<uuid::Uuid>>>,
}

impl SessionSlot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns false when a session is already active.
    pub fn claim(&self, session_id: uuid::Uuid) -> bool {
        let mut slot = self.inner.lock().expect("session slot poisoned");
        if slot.is_some() {
            return false;
        }
        *slot = Some(session_id);
        true
    }

    pub fn release(&self, session_id: uuid::Uuid) {
        let mut slot = self.inner.lock().expect("session slot poisoned");
        if *slot == Some(session_id) {
            *slot = None;
        }
    }

    pub fn release_any(&self) {
        *self.inner.lock().expect("session slot poisoned") = None;
    }

    pub fn active(&self) -> Option<uuid::Uuid> {
        *self.inner.lock().expect("session slot poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_session_slot_admits_one_at_a_time() {
        let slot = SessionSlot::new();
        let first = uuid::Uuid::new_v4();
        let second = uuid::Uuid::new_v4();

        assert!(slot.claim(first));
        assert!(!slot.claim(second), "a second session must be refused");
        assert_eq!(slot.active(), Some(first));

        // Releasing someone else's session must not free the slot.
        slot.release(second);
        assert_eq!(slot.active(), Some(first));

        slot.release(first);
        assert!(slot.claim(second));
    }

    #[cfg(unix)]
    #[test]
    fn spawns_a_shell_and_reads_its_output() {
        let (mut session, mut reader) =
            PtySession::spawn("/bin/sh", &[], None, 80, 24).expect("sh should start");

        session.write_input(b"echo termy-pty-ok\n").unwrap();

        let mut seen = String::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut buf = [0u8; 4096];

        while std::time::Instant::now() < deadline {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    seen.push_str(&String::from_utf8_lossy(&buf[..n]));
                    if seen.contains("termy-pty-ok") {
                        break;
                    }
                }
                Err(_) => break,
            }
        }

        assert!(
            seen.contains("termy-pty-ok"),
            "expected the echo to come back, saw: {seen:?}"
        );
        session.terminate();
    }

    #[cfg(unix)]
    #[test]
    fn terminating_kills_the_whole_process_group() {
        use std::io::Read;

        let (mut session, mut reader) =
            PtySession::spawn("/bin/sh", &[], None, 80, 24).expect("sh should start");

        // Start a grandchild that would outlive a naive kill of the shell, and
        // have it print its pid so the test can check on it afterwards.
        session
            .write_input(b"sleep 300 & echo GRANDCHILD=$!\n")
            .unwrap();

        // The pty echoes the input line back, so "GRANDCHILD=$!" appears before
        // the shell has run anything. Only an occurrence followed by digits is
        // the real output.
        let mut seen = String::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut buf = [0u8; 4096];
        let mut pid = None;

        while std::time::Instant::now() < deadline && pid.is_none() {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    seen.push_str(&String::from_utf8_lossy(&buf[..n]));
                    pid = extract_pid(&seen);
                }
                Err(_) => break,
            }
        }

        let pid: i32 =
            pid.unwrap_or_else(|| panic!("could not read the grandchild pid from {seen:?}"));

        assert!(
            process_alive(pid),
            "the grandchild should be running before teardown"
        );

        session.terminate();

        // The group signal should have reached it.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while std::time::Instant::now() < deadline && process_alive(pid) {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        assert!(
            !process_alive(pid),
            "sleep {pid} survived teardown; killing only the shell orphans its children"
        );
    }

    #[cfg(unix)]
    fn extract_pid(seen: &str) -> Option<i32> {
        seen.split("GRANDCHILD=").skip(1).find_map(|rest| {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            digits.parse().ok()
        })
    }

    #[cfg(unix)]
    fn process_alive(pid: i32) -> bool {
        std::path::Path::new(&format!("/proc/{pid}")).exists()
    }
}
