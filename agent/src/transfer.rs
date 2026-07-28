//! Receiving a note and its attachments (doc 8.4, 10.3, 10.4).
//!
//! MVP writes straight into the receive root with no staging area, so a failure
//! can leave finished files and one partial file behind. That is a documented
//! trade-off (doc 4.8), and the reason `transfer.result` carries a warning the
//! UI is required to surface.

use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::paths;
use crate::AgentError;

/// Doc 4.12 and 8.4.
pub const MAX_FILES: usize = 256;
pub const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_TRANSFER_BYTES: u64 = 256 * 1024 * 1024;
/// Doc 8.6: top the credit window up after this much lands on disk.
pub const CREDIT_STEP: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub index: usize,
    pub relative_path: String,
    pub size: u64,
}

struct OpenFile {
    index: usize,
    handle: File,
    written: u64,
    path: PathBuf,
}

pub struct TransferSession {
    receive_root: PathBuf,
    entries: Vec<Entry>,
    current: Option<OpenFile>,
    finished: HashSet<usize>,
    /// Cumulative bytes accepted, used to decide when to grant more credit.
    accepted_bytes: u64,
    granted_bytes: u64,
    completed: bool,
}

impl TransferSession {
    /// Validates the manifest and prepares to receive. Every structural rule is
    /// checked here so a hostile manifest is rejected before any file is
    /// created, not partway through.
    pub fn new(
        receive_root: PathBuf,
        entries: Vec<Entry>,
        root_note: &str,
        initial_credit: u64,
    ) -> Result<Self, AgentError> {
        if entries.is_empty() {
            return Err(AgentError::Transfer("manifest is empty".into()));
        }
        if entries.len() > MAX_FILES {
            return Err(AgentError::Transfer(format!(
                "manifest has {} files, the limit is {MAX_FILES}",
                entries.len()
            )));
        }

        let mut seen = HashSet::new();
        let mut total = 0u64;

        for (position, entry) in entries.iter().enumerate() {
            if entry.index != position {
                return Err(AgentError::Transfer(format!(
                    "entry {position} declares index {}, indices must run 0..n-1",
                    entry.index
                )));
            }
            if entry.size > MAX_FILE_BYTES {
                return Err(AgentError::Transfer(format!(
                    "{} is {} bytes, over the 64 MiB per-file limit",
                    entry.relative_path, entry.size
                )));
            }
            if !seen.insert(entry.relative_path.clone()) {
                return Err(AgentError::Transfer(format!(
                    "duplicate path {}",
                    entry.relative_path
                )));
            }

            // Authoritative path check (doc 10.3.7): reject the whole batch
            // rather than silently renaming anything.
            paths::resolve_under_root(&receive_root, &entry.relative_path)
                .map_err(|e| AgentError::Transfer(format!("{}: {e}", entry.relative_path)))?;

            total = total.saturating_add(entry.size);
        }

        if total > MAX_TRANSFER_BYTES {
            return Err(AgentError::Transfer(format!(
                "manifest totals {total} bytes, over the 256 MiB limit"
            )));
        }

        if entries[0].relative_path != root_note {
            return Err(AgentError::Transfer(
                "rootNote must be the first entry (doc 10.1)".into(),
            ));
        }

        Ok(Self {
            receive_root,
            entries,
            current: None,
            finished: HashSet::new(),
            accepted_bytes: 0,
            granted_bytes: initial_credit,
            completed: false,
        })
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Writes one chunk. Returns the new cumulative credit when the window
    /// should be topped up, so the caller can emit `transfer.credit`.
    pub fn write_chunk(
        &mut self,
        file_index: usize,
        offset: u64,
        data: &[u8],
    ) -> Result<Option<u64>, AgentError> {
        let entry = self
            .entries
            .get(file_index)
            .ok_or_else(|| AgentError::Transfer(format!("unknown fileIndex {file_index}")))?
            .clone();

        if self.finished.contains(&file_index) {
            return Err(AgentError::Transfer(format!(
                "fileIndex {file_index} already ended"
            )));
        }

        // Switching files implicitly closes the previous one only if it was
        // properly ended; otherwise the peer skipped a fileEnd.
        if let Some(open) = &self.current {
            if open.index != file_index {
                return Err(AgentError::Transfer(format!(
                    "chunk for fileIndex {file_index} arrived while {} is still open",
                    open.index
                )));
            }
        }

        if self.current.is_none() {
            let path = paths::resolve_under_root(&self.receive_root, &entry.relative_path)
                .map_err(|e| AgentError::Transfer(format!("{}: {e}", entry.relative_path)))?;

            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Truncating open is the documented overwrite behaviour (doc 2.1.9).
            let handle = File::create(&path)?;
            self.current = Some(OpenFile { index: file_index, handle, written: 0, path });
        }

        let open = self.current.as_mut().expect("just ensured");

        if offset != open.written {
            return Err(AgentError::Transfer(format!(
                "fileIndex {file_index} expected offset {} but got {offset}",
                open.written
            )));
        }

        let next = open.written + data.len() as u64;
        if next > entry.size {
            return Err(AgentError::Transfer(format!(
                "fileIndex {file_index} would exceed its declared size {}",
                entry.size
            )));
        }

        open.handle.write_all(data)?;
        open.written = next;

        self.accepted_bytes += data.len() as u64;

        // Grant more only once a whole step has landed, so a chatty transfer
        // does not produce one control message per chunk.
        if self.accepted_bytes + CREDIT_STEP >= self.granted_bytes {
            self.granted_bytes = self.accepted_bytes + CREDIT_STEP.max(self.granted_bytes / 2);
            return Ok(Some(self.granted_bytes));
        }

        Ok(None)
    }

    /// Doc 10.4: `sentSize` is authoritative for the success check, and an empty
    /// file produces no chunk at all, so the file must be created here.
    pub fn finish_file(&mut self, file_index: usize, sent_size: u64) -> Result<(), AgentError> {
        let entry = self
            .entries
            .get(file_index)
            .ok_or_else(|| AgentError::Transfer(format!("unknown fileIndex {file_index}")))?
            .clone();

        if !self.finished.insert(file_index) {
            return Err(AgentError::Transfer(format!("fileIndex {file_index} ended twice")));
        }

        if sent_size > entry.size {
            return Err(AgentError::Transfer(format!(
                "fileIndex {file_index} sent {sent_size} bytes, over the declared {}",
                entry.size
            )));
        }

        match self.current.take() {
            Some(mut open) if open.index == file_index => {
                if open.written != sent_size {
                    return Err(AgentError::Transfer(format!(
                        "fileIndex {file_index} wrote {} bytes but declared {sent_size}",
                        open.written
                    )));
                }
                open.handle.flush()?;
                open.handle.sync_all()?;
                drop(open.handle);
            }
            Some(open) => {
                return Err(AgentError::Transfer(format!(
                    "fileEnd for {file_index} while {} is open",
                    open.index
                )))
            }
            None => {
                if sent_size != 0 {
                    return Err(AgentError::Transfer(format!(
                        "fileEnd for {file_index} claims {sent_size} bytes but no chunk arrived"
                    )));
                }
                // Empty file: no chunk was ever sent, so create it now.
                let path = paths::resolve_under_root(&self.receive_root, &entry.relative_path)
                    .map_err(|e| AgentError::Transfer(format!("{}: {e}", entry.relative_path)))?;
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                File::create(&path)?;
            }
        }

        Ok(())
    }

    /// Doc 10.4: success requires every manifest file to have ended.
    pub fn complete(&mut self) -> Result<(), AgentError> {
        if self.current.is_some() {
            return Err(AgentError::Transfer("a file is still open".into()));
        }
        if self.finished.len() != self.entries.len() {
            return Err(AgentError::Transfer(format!(
                "only {} of {} files completed",
                self.finished.len(),
                self.entries.len()
            )));
        }
        self.completed = true;
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        self.completed
    }

    /// Closes any open handle. The partial file is deliberately left on disk:
    /// doc 4.8 accepts that, and the UI warns about it.
    pub fn abort(&mut self) {
        if let Some(open) = self.current.take() {
            tracing::warn!(path = %open.path.display(), "transfer aborted with a partial file");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(index: usize, path: &str, size: u64) -> Entry {
        Entry { index, relative_path: path.into(), size }
    }

    fn session(root: &std::path::Path, entries: Vec<Entry>) -> TransferSession {
        let root_note = entries[0].relative_path.clone();
        TransferSession::new(root.to_path_buf(), entries, &root_note, 4 * 1024 * 1024).unwrap()
    }

    #[test]
    fn writes_a_note_and_its_attachment() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = session(
            dir.path(),
            vec![entry(0, "notes/demo.md", 11), entry(1, "assets/img.png", 4)],
        );

        s.write_chunk(0, 0, b"hello world").unwrap();
        s.finish_file(0, 11).unwrap();
        s.write_chunk(1, 0, b"\x89PNG").unwrap();
        s.finish_file(1, 4).unwrap();
        s.complete().unwrap();

        assert!(s.is_complete());
        assert_eq!(std::fs::read(dir.path().join("notes/demo.md")).unwrap(), b"hello world");
        assert_eq!(std::fs::read(dir.path().join("assets/img.png")).unwrap(), b"\x89PNG");
    }

    #[test]
    fn an_empty_file_is_still_created() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = session(dir.path(), vec![entry(0, "empty.md", 0)]);

        // No chunk at all, per doc 10.4.
        s.finish_file(0, 0).unwrap();
        s.complete().unwrap();

        let path = dir.path().join("empty.md");
        assert!(path.exists(), "a zero-byte file must still be created");
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 0);
    }

    #[test]
    fn an_existing_file_is_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("demo.md"), b"old content that is longer").unwrap();

        let mut s = session(dir.path(), vec![entry(0, "demo.md", 3)]);
        s.write_chunk(0, 0, b"new").unwrap();
        s.finish_file(0, 3).unwrap();
        s.complete().unwrap();

        assert_eq!(std::fs::read(dir.path().join("demo.md")).unwrap(), b"new");
    }

    #[test]
    fn chunks_must_be_contiguous() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = session(dir.path(), vec![entry(0, "a.md", 10)]);

        s.write_chunk(0, 0, b"abc").unwrap();
        assert!(s.write_chunk(0, 5, b"def").is_err(), "a gap must be rejected");
        assert!(s.write_chunk(0, 3, b"def").is_ok());
    }

    #[test]
    fn a_file_cannot_exceed_its_declared_size() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = session(dir.path(), vec![entry(0, "a.md", 4)]);
        assert!(s.write_chunk(0, 0, b"toolong").is_err());
    }

    #[test]
    fn sent_size_must_match_what_was_written() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = session(dir.path(), vec![entry(0, "a.md", 10)]);
        s.write_chunk(0, 0, b"abc").unwrap();
        assert!(s.finish_file(0, 5).is_err(), "declared size must match the bytes written");
    }

    #[test]
    fn completion_requires_every_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = session(dir.path(), vec![entry(0, "a.md", 1), entry(1, "b.md", 1)]);

        s.write_chunk(0, 0, b"a").unwrap();
        s.finish_file(0, 1).unwrap();
        assert!(s.complete().is_err(), "b.md never arrived");

        s.write_chunk(1, 0, b"b").unwrap();
        s.finish_file(1, 1).unwrap();
        assert!(s.complete().is_ok());
    }

    #[test]
    fn manifests_are_validated_before_anything_is_written() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().to_path_buf();

        // Index gap.
        assert!(TransferSession::new(
            root.clone(),
            vec![entry(0, "a.md", 1), entry(2, "b.md", 1)],
            "a.md",
            1024
        )
        .is_err());

        // rootNote mismatch.
        assert!(
            TransferSession::new(root.clone(), vec![entry(0, "a.md", 1)], "other.md", 1024).is_err()
        );

        // Duplicate path.
        assert!(TransferSession::new(
            root.clone(),
            vec![entry(0, "a.md", 1), entry(1, "a.md", 1)],
            "a.md",
            1024
        )
        .is_err());

        // Path traversal.
        assert!(TransferSession::new(
            root.clone(),
            vec![entry(0, "../escape.md", 1)],
            "../escape.md",
            1024
        )
        .is_err());

        // Over the per-file limit.
        assert!(TransferSession::new(
            root.clone(),
            vec![entry(0, "big.bin", MAX_FILE_BYTES + 1)],
            "big.bin",
            1024
        )
        .is_err());

        // Over the whole-transfer limit.
        let many: Vec<Entry> = (0..5)
            .map(|i| entry(i, &format!("f{i}.bin"), MAX_FILE_BYTES))
            .collect();
        assert!(TransferSession::new(root, many, "f0.bin", 1024).is_err());

        // Nothing should have been created by any of those attempts.
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    fn credit_is_granted_as_bytes_land() {
        let dir = tempfile::tempdir().unwrap();
        let mut s = session(dir.path(), vec![entry(0, "big.bin", 8 * 1024 * 1024)]);

        let chunk = vec![0u8; 256 * 1024];
        let mut grants = Vec::new();
        let mut offset = 0u64;

        for _ in 0..24 {
            if let Some(granted) = s.write_chunk(0, offset, &chunk).unwrap() {
                grants.push(granted);
            }
            offset += chunk.len() as u64;
        }

        assert!(!grants.is_empty(), "the window must be topped up during a long transfer");
        assert!(
            grants.windows(2).all(|w| w[1] > w[0]),
            "granted totals must increase monotonically"
        );
    }
}
