//! Single-instance lock (doc 7.6).
//!
//! Not optional. `systemd`'s `Restart=always` plus doc 8.2's "newest agent
//! connection wins" means two live instances would take turns evicting each
//! other from the relay forever, and the device would look like it is flapping
//! rather than like it is misconfigured.

use std::path::{Path, PathBuf};

use crate::AgentError;

pub struct InstanceLock {
    #[allow(dead_code)]
    file: std::fs::File,
    path: PathBuf,
}

impl InstanceLock {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// `$XDG_RUNTIME_DIR/termesh-agent.lock` when the runtime dir exists, otherwise
/// next to the config. The runtime dir is preferred because it is cleared on
/// reboot, so a stale file can never outlive the machine it described.
pub fn lock_path() -> PathBuf {
    if let Some(runtime) = std::env::var_os("XDG_RUNTIME_DIR").filter(|v| !v.is_empty()) {
        return PathBuf::from(runtime).join("termesh-agent.lock");
    }
    crate::config::config_dir().join("agent.lock")
}

#[cfg(unix)]
pub fn acquire(path: &Path) -> Result<InstanceLock, AgentError> {
    use std::os::unix::io::AsRawFd;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)?;

    // flock is released automatically when the process dies for any reason,
    // including SIGKILL, so a crashed agent never leaves the lock held.
    let rc = unsafe { libc_flock(file.as_raw_fd(), LOCK_EX | LOCK_NB) };
    if rc != 0 {
        return Err(AgentError::AlreadyRunning(path.display().to_string()));
    }

    Ok(InstanceLock {
        file,
        path: path.to_path_buf(),
    })
}

#[cfg(unix)]
const LOCK_EX: i32 = 2;
#[cfg(unix)]
const LOCK_NB: i32 = 4;

#[cfg(unix)]
extern "C" {
    #[link_name = "flock"]
    fn libc_flock(fd: i32, operation: i32) -> i32;
}

/// Windows has no flock; an exclusive open of the lock file gives the same
/// mutual exclusion and is released by the kernel when the process exits.
#[cfg(windows)]
pub fn acquire(path: &Path) -> Result<InstanceLock, AgentError> {
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_SHARE_NONE: u32 = 0;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = std::fs::OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .share_mode(FILE_SHARE_NONE)
        .open(path)
        .map_err(|_| AgentError::AlreadyRunning(path.display().to_string()))?;

    Ok(InstanceLock {
        file,
        path: path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_acquisition_is_refused_while_the_first_is_held() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("agent.lock");

        let first = acquire(&path).expect("first lock should succeed");

        match acquire(&path) {
            Err(AgentError::AlreadyRunning(_)) => {}
            Err(other) => panic!("expected AlreadyRunning, got {other}"),
            Ok(_) => panic!("a second lock must not be granted while the first is held"),
        }

        drop(first);
        acquire(&path).expect("the lock should be free once the holder drops it");
    }

    #[test]
    fn the_lock_file_is_created_with_its_parent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("agent.lock");
        let lock = acquire(&path).unwrap();
        assert!(lock.path().exists());
    }
}
