//! Receive-path safety (doc 10.3).
//!
//! The plugin pre-checks the same rules, but this side is authoritative: the
//! agent is what actually writes to disk, and it must assume the manifest came
//! from a hostile peer.
//!
//! Structural checks (no `..`, no absolute form, no NUL) are shared; the
//! Windows-only rules are applied unconditionally on Windows and skipped on
//! Unix, because a name like `aux.txt` is perfectly legal on ext4 and refusing
//! it there would reject valid vaults.

use std::path::{Component, Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub enum PathError {
    Empty,
    Absolute,
    Traversal,
    EmptySegment,
    NulByte,
    Backslash,
    TooLong,
    WindowsReservedName(String),
    WindowsIllegalChar,
    WindowsTrailingDotOrSpace,
    EscapesRoot,
    SymlinkEscape,
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "path is empty"),
            Self::Absolute => write!(f, "path is absolute or has a drive/UNC root"),
            Self::Traversal => write!(f, "path contains a '..' segment"),
            Self::EmptySegment => write!(f, "path contains an empty segment"),
            Self::NulByte => write!(f, "path contains a NUL byte"),
            Self::Backslash => write!(f, "path contains a backslash; '/' is the only separator"),
            Self::TooLong => write!(f, "path exceeds 1024 UTF-8 bytes"),
            Self::WindowsReservedName(n) => write!(f, "'{n}' is a reserved device name on Windows"),
            Self::WindowsIllegalChar => write!(f, "path contains a character Windows forbids"),
            Self::WindowsTrailingDotOrSpace => {
                write!(
                    f,
                    "a path segment ends with a dot or space, which Windows strips"
                )
            }
            Self::EscapesRoot => write!(f, "resolved path escapes the receive root"),
            Self::SymlinkEscape => write!(f, "an existing symlink leads outside the receive root"),
        }
    }
}

const MAX_PATH_BYTES: usize = 1024;

/// Names Windows treats as devices regardless of extension.
const WINDOWS_RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Validates the protocol form of a vault-relative path.
pub fn validate_relative(path: &str) -> Result<(), PathError> {
    if path.is_empty() {
        return Err(PathError::Empty);
    }
    if path.len() > MAX_PATH_BYTES {
        return Err(PathError::TooLong);
    }
    if path.contains('\0') {
        return Err(PathError::NulByte);
    }
    if path.contains('\\') {
        return Err(PathError::Backslash);
    }
    if path.starts_with('/') {
        return Err(PathError::Absolute);
    }
    // C:/... and C:\... alike.
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return Err(PathError::Absolute);
    }

    for segment in path.split('/') {
        if segment.is_empty() {
            return Err(PathError::EmptySegment);
        }
        if segment == ".." {
            return Err(PathError::Traversal);
        }
        if segment == "." {
            return Err(PathError::EmptySegment);
        }
    }

    if cfg!(windows) {
        validate_windows(path)?;
    }

    Ok(())
}

/// Windows-only naming rules (doc 10.3.5).
pub fn validate_windows(path: &str) -> Result<(), PathError> {
    for segment in path.split('/') {
        if segment.ends_with('.') || segment.ends_with(' ') {
            return Err(PathError::WindowsTrailingDotOrSpace);
        }
        if segment
            .chars()
            .any(|c| matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*'))
        {
            return Err(PathError::WindowsIllegalChar);
        }
        if segment.chars().any(|c| (c as u32) < 0x20) {
            return Err(PathError::WindowsIllegalChar);
        }

        let stem = segment
            .split('.')
            .next()
            .unwrap_or(segment)
            .to_ascii_uppercase();
        if WINDOWS_RESERVED.contains(&stem.as_str()) {
            return Err(PathError::WindowsReservedName(stem));
        }
    }
    Ok(())
}

/// Resolves a validated relative path under `receive_root` and proves the result
/// stays inside it.
///
/// Two separate defences: the lexical check catches a malformed manifest, and
/// the canonical check catches an existing symlink pointing elsewhere. The
/// second matters because the attacker may control the target filesystem's
/// existing contents even when they cannot control the manifest.
pub fn resolve_under_root(receive_root: &Path, relative: &str) -> Result<PathBuf, PathError> {
    validate_relative(relative)?;

    let mut resolved = receive_root.to_path_buf();
    for segment in relative.split('/') {
        resolved.push(segment);
    }

    // Lexical containment: no component may climb out.
    let mut depth = 0i32;
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::ParentDir => depth -= 1,
            Component::RootDir | Component::Prefix(_) => return Err(PathError::Absolute),
        }
        if depth < 0 {
            return Err(PathError::EscapesRoot);
        }
    }

    // Symlink containment: check every existing ancestor, since a link anywhere
    // along the way is enough to redirect the write.
    let root_real = receive_root
        .canonicalize()
        .map_err(|_| PathError::EscapesRoot)?;
    let mut probe = resolved.clone();
    loop {
        if probe.exists() {
            let real = probe.canonicalize().map_err(|_| PathError::SymlinkEscape)?;
            if !real.starts_with(&root_real) {
                return Err(PathError::SymlinkEscape);
            }
            break;
        }
        match probe.parent() {
            Some(parent) if parent.starts_with(receive_root) => probe = parent.to_path_buf(),
            _ => break,
        }
    }

    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ordinary_vault_paths() {
        for path in [
            "a.md",
            "notes/demo.md",
            "assets/img/diagram.png",
            "a b/c d.md",
            "中文/笔记.md",
        ] {
            assert_eq!(validate_relative(path), Ok(()), "{path} should be accepted");
        }
    }

    #[test]
    fn rejects_structural_attacks() {
        assert_eq!(validate_relative(""), Err(PathError::Empty));
        assert_eq!(validate_relative("/etc/passwd"), Err(PathError::Absolute));
        assert_eq!(
            validate_relative("C:/Windows/system.ini"),
            Err(PathError::Absolute)
        );
        assert_eq!(validate_relative("../escape.md"), Err(PathError::Traversal));
        assert_eq!(
            validate_relative("notes/../../etc/shadow"),
            Err(PathError::Traversal)
        );
        assert_eq!(
            validate_relative("notes//demo.md"),
            Err(PathError::EmptySegment)
        );
        assert_eq!(
            validate_relative("notes/./demo.md"),
            Err(PathError::EmptySegment)
        );
        assert_eq!(
            validate_relative("notes\\demo.md"),
            Err(PathError::Backslash)
        );
        assert_eq!(validate_relative("a\0b.md"), Err(PathError::NulByte));
        assert_eq!(
            validate_relative(&"x".repeat(1025)),
            Err(PathError::TooLong)
        );
    }

    #[test]
    fn windows_rules_are_explicit_regardless_of_host() {
        assert_eq!(
            validate_windows("CON.txt"),
            Err(PathError::WindowsReservedName("CON".into()))
        );
        assert_eq!(
            validate_windows("nul"),
            Err(PathError::WindowsReservedName("NUL".into()))
        );
        assert_eq!(
            validate_windows("a/COM1.md"),
            Err(PathError::WindowsReservedName("COM1".into()))
        );
        assert_eq!(
            validate_windows("trailing."),
            Err(PathError::WindowsTrailingDotOrSpace)
        );
        assert_eq!(
            validate_windows("trailing "),
            Err(PathError::WindowsTrailingDotOrSpace)
        );
        assert_eq!(
            validate_windows("what?.md"),
            Err(PathError::WindowsIllegalChar)
        );
        assert_eq!(
            validate_windows("a:b.md"),
            Err(PathError::WindowsIllegalChar)
        );

        // Names that merely start with a reserved word are fine.
        assert_eq!(validate_windows("console.md"), Ok(()));
        assert_eq!(validate_windows("nullable.md"), Ok(()));
    }

    #[test]
    fn resolves_inside_the_receive_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let resolved = resolve_under_root(root, "notes/demo.md").unwrap();
        assert_eq!(resolved, root.join("notes").join("demo.md"));
        assert!(resolved.starts_with(root));
    }

    #[test]
    fn refuses_to_resolve_outside_the_root() {
        let dir = tempfile::tempdir().unwrap();
        assert!(resolve_under_root(dir.path(), "../outside.md").is_err());
        assert!(resolve_under_root(dir.path(), "/etc/passwd").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn refuses_a_path_that_traverses_an_existing_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();

        // receive_root/escape -> /somewhere/else
        std::os::unix::fs::symlink(outside.path(), dir.path().join("escape")).unwrap();

        let err = resolve_under_root(dir.path(), "escape/loot.md").unwrap_err();
        assert_eq!(err, PathError::SymlinkEscape);

        // A normal subdirectory next to it still works.
        std::fs::create_dir(dir.path().join("ok")).unwrap();
        assert!(resolve_under_root(dir.path(), "ok/file.md").is_ok());
    }
}
