//! Multi-session PTY table (implementation doc §7.5).
//!
//! v2.0 replaces V1's `pty::SessionSlot` - which admits exactly one active
//! session per agent - with a `HashMap<SessionId, SessionHandle>` capped at
//! a configurable `maxConcurrentSessions` (doc §7.3). Each entry pairs a
//! `PtySession` with the `lastKnownCwd` doc §7.6 needs to resolve where a
//! dropped note lands: cwd tracking is owned here (updated from
//! `termstream::ShellEventPayload.cwd` once that wiring exists) rather than
//! duplicated per call site.
//!
//! Deliberately not wired to any transport yet: opening a session still
//! requires a `PtySession` the caller already spawned. The `termy/terminal/1`
//! accept loop that spawns one per incoming stream is transport-layer work
//! blocked on the A0 spike, same as `termstream`.

use std::collections::HashMap;
use std::path::PathBuf;

use uuid::Uuid;

use crate::pty::PtySession;
use crate::termstream::ShellEventPayload;
use crate::AgentError;

pub struct SessionHandle {
    pub session_id: Uuid,
    pub pty: PtySession,
    pub last_known_cwd: Option<PathBuf>,
}

/// Bounded by `max_concurrent` (doc §7.3's `maxConcurrentSessions`); a
/// request past that returns `AgentError::SessionLimitReached` rather than
/// queuing, matching doc §7.5's "reject, don't queue" behaviour.
pub struct SessionTable {
    sessions: HashMap<Uuid, SessionHandle>,
    max_concurrent: usize,
}

impl SessionTable {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            sessions: HashMap::new(),
            max_concurrent,
        }
    }

    pub fn open(&mut self, session_id: Uuid, pty: PtySession) -> Result<&mut SessionHandle, AgentError> {
        if self.sessions.contains_key(&session_id) {
            return Err(AgentError::Protocol(format!(
                "session {session_id} is already open"
            )));
        }
        if self.sessions.len() >= self.max_concurrent {
            return Err(AgentError::SessionLimitReached(self.max_concurrent));
        }

        let handle = SessionHandle {
            session_id,
            pty,
            last_known_cwd: None,
        };
        Ok(self.sessions.entry(session_id).or_insert(handle))
    }

    /// Removes and returns the session so the caller can run teardown (or
    /// just let it drop - `PtySession::drop` already terminates the shell).
    pub fn close(&mut self, session_id: Uuid) -> Option<SessionHandle> {
        self.sessions.remove(&session_id)
    }

    pub fn get_mut(&mut self, session_id: Uuid) -> Option<&mut SessionHandle> {
        self.sessions.get_mut(&session_id)
    }

    pub fn contains(&self, session_id: Uuid) -> bool {
        self.sessions.contains_key(&session_id)
    }

    /// No-op for an unknown `session_id`: a shell event can race a session's
    /// own close (the PTY exits and is torn down before its last OSC event
    /// is processed), and that race is not an error.
    pub fn set_cwd(&mut self, session_id: Uuid, cwd: PathBuf) {
        if let Some(handle) = self.sessions.get_mut(&session_id) {
            handle.last_known_cwd = Some(cwd);
        }
    }

    /// Doc §7.6: every `shellEvent` frame that carries a `cwd` updates that
    /// session's `lastKnownCwd`. Events without a `cwd` (most of them - only
    /// the events a future cwd-reporting mechanism actually attaches one to
    /// will set it) leave the last known value untouched rather than
    /// clearing it, since "no cwd on this particular event" does not mean
    /// "the shell's directory is now unknown".
    pub fn apply_shell_event(&mut self, session_id: Uuid, event: &ShellEventPayload) {
        if let Some(cwd) = &event.cwd {
            self.set_cwd(session_id, PathBuf::from(cwd));
        }
    }

    pub fn count(&self) -> usize {
        self.sessions.len()
    }

    pub fn ids(&self) -> impl Iterator<Item = &Uuid> {
        self.sessions.keys()
    }

    /// All sessions removed, in unspecified order - used when the single
    /// control-end connection drops (doc §7.5: "该连接整体断开时，其下所有会话一并终止").
    pub fn close_all(&mut self) -> Vec<SessionHandle> {
        self.sessions.drain().map(|(_, handle)| handle).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_session() -> PtySession {
        #[cfg(unix)]
        let (session, reader) = PtySession::spawn("/bin/sh", &[], None, 80, 24).expect("sh should start");
        #[cfg(windows)]
        let (session, reader) =
            PtySession::spawn("cmd.exe", &[], None, 80, 24).expect("cmd should start");

        // Drain the pty like the real serve loop's output pump does. An
        // undrained ConPTY pipe can wedge child teardown on Windows - the
        // 2026-07-31 acceptance run hung exactly here when the reader was
        // silently dropped.
        std::thread::spawn(move || {
            use std::io::Read;
            let mut reader = reader;
            let mut buf = [0u8; 4096];
            while matches!(reader.read(&mut buf), Ok(n) if n > 0) {}
        });

        session
    }

    #[test]
    fn opens_and_closes_a_session() {
        let mut table = SessionTable::new(4);
        let id = Uuid::new_v4();

        table.open(id, spawn_session()).unwrap();
        assert_eq!(table.count(), 1);
        assert!(table.contains(id));

        let closed = table.close(id).unwrap();
        assert_eq!(closed.session_id, id);
        assert_eq!(table.count(), 0);
        assert!(table.close(id).is_none(), "closing twice is a no-op");
    }

    #[test]
    fn two_sessions_are_independent() {
        let mut table = SessionTable::new(4);
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();

        table.open(first, spawn_session()).unwrap();
        table.open(second, spawn_session()).unwrap();
        assert_eq!(table.count(), 2);

        table.set_cwd(first, PathBuf::from("/tmp/first"));
        assert_eq!(
            table.get_mut(first).unwrap().last_known_cwd,
            Some(PathBuf::from("/tmp/first"))
        );
        assert_eq!(table.get_mut(second).unwrap().last_known_cwd, None);

        table.close(first);
        assert_eq!(table.count(), 1);
        assert!(table.contains(second), "closing one must not affect the other");
    }

    #[test]
    fn opening_the_same_session_id_twice_is_rejected() {
        let mut table = SessionTable::new(4);
        let id = Uuid::new_v4();
        table.open(id, spawn_session()).unwrap();
        assert!(table.open(id, spawn_session()).is_err());
        assert_eq!(table.count(), 1);
    }

    #[test]
    fn a_request_past_max_concurrent_is_rejected_not_queued() {
        let mut table = SessionTable::new(1);
        table.open(Uuid::new_v4(), spawn_session()).unwrap();

        let rejected = table.open(Uuid::new_v4(), spawn_session());
        assert!(matches!(
            rejected,
            Err(AgentError::SessionLimitReached(1))
        ));
        assert_eq!(table.count(), 1, "the rejected session must not be admitted");
    }

    #[test]
    fn set_cwd_on_an_unknown_session_is_a_harmless_no_op() {
        let mut table = SessionTable::new(4);
        table.set_cwd(Uuid::new_v4(), PathBuf::from("/tmp/ghost"));
    }

    #[test]
    fn close_all_empties_the_table() {
        let mut table = SessionTable::new(4);
        table.open(Uuid::new_v4(), spawn_session()).unwrap();
        table.open(Uuid::new_v4(), spawn_session()).unwrap();

        let closed = table.close_all();
        assert_eq!(closed.len(), 2);
        assert_eq!(table.count(), 0);
    }

    #[test]
    fn a_shell_event_without_cwd_leaves_the_last_known_value_untouched() {
        let mut table = SessionTable::new(4);
        let id = Uuid::new_v4();
        table.open(id, spawn_session()).unwrap();

        table.apply_shell_event(
            id,
            &crate::termstream::ShellEventPayload {
                event: "command_end".into(),
                source: Some("osc133".into()),
                cwd: Some("/home/user/project".into()),
                exit_code: Some(0),
            },
        );
        assert_eq!(
            table.get_mut(id).unwrap().last_known_cwd,
            Some(PathBuf::from("/home/user/project"))
        );

        table.apply_shell_event(
            id,
            &crate::termstream::ShellEventPayload {
                event: "prompt_start".into(),
                source: None,
                cwd: None,
                exit_code: None,
            },
        );
        assert_eq!(
            table.get_mut(id).unwrap().last_known_cwd,
            Some(PathBuf::from("/home/user/project")),
            "an event with no cwd must not clear a previously known one"
        );
    }

    #[cfg(unix)]
    #[test]
    fn closing_a_session_through_the_table_kills_its_whole_process_tree() {
        use std::io::Read;

        let (session, mut reader) =
            PtySession::spawn("/bin/sh", &[], None, 80, 24).expect("sh should start");

        let mut table = SessionTable::new(4);
        let id = Uuid::new_v4();
        table.open(id, session).unwrap();

        table
            .get_mut(id)
            .unwrap()
            .pty
            .write_input(b"sleep 300 & echo GRANDCHILD=$!\n")
            .unwrap();

        let mut seen = String::new();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let mut buf = [0u8; 4096];
        let mut pid = None;
        while std::time::Instant::now() < deadline && pid.is_none() {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    seen.push_str(&String::from_utf8_lossy(&buf[..n]));
                    // Try every occurrence, not just the first: the pty echoes
                    // the input line back, so "GRANDCHILD=$!" (no digits)
                    // appears before the shell's real "GRANDCHILD=<pid>"
                    // output. Same trap pty.rs's extract_pid documents.
                    pid = seen.split("GRANDCHILD=").skip(1).find_map(|rest| {
                        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
                        digits.parse::<i32>().ok()
                    });
                }
                Err(_) => break,
            }
        }
        let pid = pid.unwrap_or_else(|| panic!("could not read the grandchild pid from {seen:?}"));
        assert!(
            std::path::Path::new(&format!("/proc/{pid}")).exists(),
            "the grandchild should be running before the session is closed"
        );

        // Closing through the table, not calling `.terminate()` directly, is
        // the point of this test: SessionTable::close() must not just drop
        // the bookkeeping entry and leave teardown to chance.
        drop(table.close(id));

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while std::time::Instant::now() < deadline && std::path::Path::new(&format!("/proc/{pid}")).exists() {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert!(
            !std::path::Path::new(&format!("/proc/{pid}")).exists(),
            "sleep {pid} survived SessionTable::close(); the whole process tree must die with the session"
        );
    }
}
