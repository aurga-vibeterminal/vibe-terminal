use std::path::{Path, PathBuf};

use vterm_common::SessionTerminalOs;
use vterm_core::folder_stream::unique_destination_path;

use super::path_listing::resolve_filesystem_path;
use super::session_error::SessionError;

pub(super) struct CreatePathRequest {
    pub(super) base_path: Option<String>,
    pub(super) path: String,
    pub(super) is_dir: bool,
    pub(super) runtime_os: SessionTerminalOs,
}

pub(super) struct DeletePathsRequest {
    pub(super) base_path: Option<String>,
    pub(super) paths: Vec<String>,
    pub(super) runtime_os: SessionTerminalOs,
}

/// Paste (copy or move) a set of source paths into a destination directory.
/// `is_cut` selects the semantics: `false` duplicates the sources (Copy+Paste),
/// `true` removes them after a successful duplicate (Cut+Paste).
pub(super) struct PastePathsRequest {
    pub(super) base_path: Option<String>,
    pub(super) src_paths: Vec<String>,
    pub(super) dst_dir: String,
    pub(super) is_cut: bool,
    pub(super) runtime_os: SessionTerminalOs,
}

pub(super) struct RenamePathRequest {
    pub(super) base_path: Option<String>,
    pub(super) old_path: String,
    pub(super) new_path: String,
    pub(super) runtime_os: SessionTerminalOs,
}

pub(super) fn create_path(request: CreatePathRequest) -> Result<PathBuf, SessionError> {
    let resolved = resolve_filesystem_path(
        request.base_path.as_deref(),
        &request.path,
        request.runtime_os,
    )
    .map_err(SessionError::FileSystemError)?;

    let Some(target) = resolved else {
        return Err(SessionError::FileSystemError(
            "Cannot create path at virtual root".to_string(),
        ));
    };

    tracing::warn!(
        path = %target.display(),
        is_dir = request.is_dir,
        "file operation: create path"
    );
    if request.is_dir {
        std::fs::create_dir_all(&target)
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    } else {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        }
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    }

    Ok(target)
}

/// Delete the requested paths. Directories are removed recursively.
/// Returns the first encountered error so the client can surface it.
pub(super) fn delete_paths(request: DeletePathsRequest) -> Result<(), SessionError> {
    let runtime_os = request.runtime_os;
    for raw in &request.paths {
        let resolved = resolve_filesystem_path(request.base_path.as_deref(), raw, runtime_os)
            .map_err(SessionError::FileSystemError)?;

        let Some(target) = resolved else {
            return Err(SessionError::FileSystemError(format!(
                "Cannot delete virtual root path: {raw}"
            )));
        };

        let metadata = match std::fs::symlink_metadata(&target) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(SessionError::FileSystemError(error.to_string()));
            }
        };

        tracing::warn!(
            path = %target.display(),
            is_dir = metadata.is_dir(),
            "file operation: delete path"
        );
        let removed = if metadata.is_dir() {
            std::fs::remove_dir_all(&target)
        } else {
            std::fs::remove_file(&target)
        };
        removed.map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    }

    Ok(())
}

/// Paste (copy or move) each `src_paths` entry into `dst_dir`.
/// `request.is_cut` selects the semantics — see `PastePathsRequest`.
///
/// Collisions auto-rename with ` copy`, ` copy 2`, … suffixes (macOS Finder /
/// Windows Explorer style). Symlinks are preserved as symlinks rather than
/// followed. Refuses pasting a path into itself or one of its descendants.
pub(super) fn paste_paths(request: PastePathsRequest) -> Result<(), SessionError> {
    let runtime_os = request.runtime_os;
    let base_path = request.base_path.as_deref();
    let dst_dir = resolve_dst_dir(base_path, &request.dst_dir, runtime_os)?;

    for raw in &request.src_paths {
        let src = resolve_source(base_path, raw, runtime_os)?;
        reject_self_target(&src, &dst_dir, raw)?;
        let file_name = src
            .file_name()
            .ok_or_else(|| SessionError::FileSystemError(format!("Invalid source path: {raw}")))?;
        let target = unique_destination_path(dst_dir.join(file_name));

        if request.is_cut {
            tracing::warn!(
                src = %src.display(),
                dst = %target.display(),
                "file operation: move path"
            );
            move_one(&src, &target)?;
        } else {
            copy_one(&src, &target)?;
        }
    }

    Ok(())
}

/// Rename `old_path` to `new_path`. Refuses to overwrite an existing
/// destination — the user explicitly chose the new name, so silent overwrite
/// would lose data. Both paths are resolved relative to `base_path`.
pub(super) fn rename_path(request: RenamePathRequest) -> Result<(), SessionError> {
    let runtime_os = request.runtime_os;
    let base_path = request.base_path.as_deref();

    let old_resolved = resolve_filesystem_path(base_path, &request.old_path, runtime_os)
        .map_err(SessionError::FileSystemError)?;
    let Some(old) = old_resolved else {
        return Err(SessionError::FileSystemError(format!(
            "Cannot rename virtual root path: {}",
            request.old_path
        )));
    };

    let new_resolved = resolve_filesystem_path(base_path, &request.new_path, runtime_os)
        .map_err(SessionError::FileSystemError)?;
    let Some(new) = new_resolved else {
        return Err(SessionError::FileSystemError(format!(
            "Cannot rename to virtual root path: {}",
            request.new_path
        )));
    };

    if old == new {
        return Ok(());
    }

    if new.exists() {
        return Err(SessionError::FileSystemError(format!(
            "Destination already exists: {}",
            request.new_path
        )));
    }

    tracing::warn!(
        old_path = %old.display(),
        new_path = %new.display(),
        "file operation: rename path"
    );
    std::fs::rename(&old, &new).map_err(|error| SessionError::FileSystemError(error.to_string()))
}

fn resolve_dst_dir(
    base_path: Option<&str>,
    dst_dir: &str,
    runtime_os: SessionTerminalOs,
) -> Result<PathBuf, SessionError> {
    let resolved = resolve_filesystem_path(base_path, dst_dir, runtime_os)
        .map_err(SessionError::FileSystemError)?;
    resolved.ok_or_else(|| {
        SessionError::FileSystemError("Cannot target virtual root as paste destination".to_string())
    })
}

fn resolve_source(
    base_path: Option<&str>,
    raw: &str,
    runtime_os: SessionTerminalOs,
) -> Result<PathBuf, SessionError> {
    let resolved = resolve_filesystem_path(base_path, raw, runtime_os)
        .map_err(SessionError::FileSystemError)?;
    resolved.ok_or_else(|| {
        SessionError::FileSystemError(format!("Cannot paste virtual root path: {raw}"))
    })
}

/// Refuse pasting a path into itself or one of its descendants — that would
/// either be a no-op or cause unbounded recursion inside `copy_one`.
fn reject_self_target(src: &Path, dst_dir: &Path, raw: &str) -> Result<(), SessionError> {
    if src == dst_dir || dst_dir.starts_with(src) {
        return Err(SessionError::FileSystemError(format!(
            "Cannot paste '{raw}' into itself or its descendant"
        )));
    }
    Ok(())
}

/// Recursively copy `src` into `dst`, preserving symlinks as symlinks
/// (Unix-only — Windows returns an error). Directories are walked and
/// recreated; regular files use `std::fs::copy`.
fn copy_one(src: &Path, dst: &Path) -> Result<(), SessionError> {
    let metadata = std::fs::symlink_metadata(src)
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    if metadata.is_symlink() {
        let target = std::fs::read_link(src)
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target, dst)
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
            return Ok(());
        }
        #[cfg(not(unix))]
        {
            let _ = target;
            return Err(SessionError::FileSystemError(
                "Symlink copy is not supported on this platform".to_string(),
            ));
        }
    }

    if metadata.is_dir() {
        std::fs::create_dir_all(dst)
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        let entries = std::fs::read_dir(src)
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        for entry in entries {
            let entry = entry.map_err(|error| SessionError::FileSystemError(error.to_string()))?;
            let entry_path = entry.path();
            let dest_path = dst.join(entry.file_name());
            copy_one(&entry_path, &dest_path)?;
        }
        return Ok(());
    }

    // Regular file (symlink_metadata distinguishes symlinks above, so this is
    // implicitly "not a symlink and not a directory").
    std::fs::copy(src, dst).map_err(|error| SessionError::FileSystemError(error.to_string()))?;
    Ok(())
}

/// Move `src` to `dst`. Uses `rename` when possible (atomic on the same
/// filesystem); falls back to `copy_one` + remove on `CrossesDevices` / EXDEV.
fn move_one(src: &Path, dst: &Path) -> Result<(), SessionError> {
    match std::fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(error) => {
            let crosses = error.kind() == std::io::ErrorKind::CrossesDevices
                || error.raw_os_error() == Some(18);
            if !crosses {
                return Err(SessionError::FileSystemError(error.to_string()));
            }
            copy_one(src, dst)?;
            let metadata = std::fs::symlink_metadata(src)
                .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
            if metadata.is_dir() {
                std::fs::remove_dir_all(src)
                    .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
            } else {
                std::fs::remove_file(src)
                    .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("vterm-file-ops-test-{id}"));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn creates_nested_directory() {
        let dir = TestDir::new();
        let result = create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "alpha/beta".to_string(),
            is_dir: true,
            runtime_os: SessionTerminalOs::Linux,
        });
        assert!(result.is_ok());
        assert!(dir.path.join("alpha/beta").is_dir());
    }

    #[test]
    fn creates_empty_file() {
        let dir = TestDir::new();
        let result = create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "note.txt".to_string(),
            is_dir: false,
            runtime_os: SessionTerminalOs::Linux,
        });
        assert!(result.is_ok());
        assert!(dir.path.join("note.txt").is_file());
    }

    #[test]
    fn create_file_refuses_overwrite_and_preserves_siblings() {
        let dir = TestDir::new();
        fs::write(dir.path.join("existing.txt"), b"original").unwrap();
        fs::write(dir.path.join("sibling.txt"), b"sibling").unwrap();

        let result = create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "existing.txt".to_string(),
            is_dir: false,
            runtime_os: SessionTerminalOs::Linux,
        });

        assert!(result.is_err());
        assert_eq!(
            fs::read(dir.path.join("existing.txt")).unwrap(),
            b"original"
        );
        assert_eq!(fs::read(dir.path.join("sibling.txt")).unwrap(), b"sibling");
    }

    #[test]
    fn deletes_file_and_directory_list() {
        let dir = TestDir::new();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "child".to_string(),
            is_dir: true,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "child/file.txt".to_string(),
            is_dir: false,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "lonely.txt".to_string(),
            is_dir: false,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();

        let result = delete_paths(DeletePathsRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            paths: vec!["child".to_string(), "lonely.txt".to_string()],
            runtime_os: SessionTerminalOs::Linux,
        });
        assert!(result.is_ok());
        assert!(!dir.path.join("child").exists());
        assert!(!dir.path.join("lonely.txt").exists());
    }

    #[test]
    fn delete_ignores_missing_paths() {
        let dir = TestDir::new();
        let result = delete_paths(DeletePathsRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            paths: vec!["ghost.txt".to_string()],
            runtime_os: SessionTerminalOs::Linux,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn paste_copy_duplicates_single_file() {
        let dir = TestDir::new();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "src.txt".to_string(),
            is_dir: false,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();
        fs::write(dir.path.join("src.txt"), b"hello").unwrap();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "dst".to_string(),
            is_dir: true,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();

        let result = paste_paths(PastePathsRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            src_paths: vec!["src.txt".to_string()],
            dst_dir: "dst".to_string(),
            is_cut: false,
            runtime_os: SessionTerminalOs::Linux,
        });
        assert!(result.is_ok());
        assert_eq!(
            fs::read(dir.path.join("dst/src.txt")).unwrap(),
            b"hello".to_vec()
        );
        // Copy is non-destructive — source remains.
        assert!(dir.path.join("src.txt").exists());
    }

    #[test]
    fn paste_copy_recurses_into_directory() {
        let dir = TestDir::new();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "tree/sub".to_string(),
            is_dir: true,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();
        fs::write(dir.path.join("tree/root.txt"), b"r").unwrap();
        fs::write(dir.path.join("tree/sub/leaf.txt"), b"l").unwrap();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "dst".to_string(),
            is_dir: true,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();

        let result = paste_paths(PastePathsRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            src_paths: vec!["tree".to_string()],
            dst_dir: "dst".to_string(),
            is_cut: false,
            runtime_os: SessionTerminalOs::Linux,
        });
        assert!(result.is_ok());
        assert!(dir.path.join("dst/tree/root.txt").is_file());
        assert_eq!(
            fs::read(dir.path.join("dst/tree/sub/leaf.txt")).unwrap(),
            b"l".to_vec()
        );
    }

    #[test]
    fn paste_copy_auto_renames_on_collision() {
        let dir = TestDir::new();
        fs::write(dir.path.join("a.txt"), b"1").unwrap();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "dst".to_string(),
            is_dir: true,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();
        fs::write(dir.path.join("dst/a.txt"), b"existing").unwrap();

        paste_paths(PastePathsRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            src_paths: vec!["a.txt".to_string()],
            dst_dir: "dst".to_string(),
            is_cut: false,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();
        // Existing file untouched; new arrival gets " copy".
        assert_eq!(fs::read(dir.path.join("dst/a.txt")).unwrap(), b"existing");
        assert_eq!(fs::read(dir.path.join("dst/a copy.txt")).unwrap(), b"1");

        // Second paste of the same source → " copy 2".
        paste_paths(PastePathsRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            src_paths: vec!["a.txt".to_string()],
            dst_dir: "dst".to_string(),
            is_cut: false,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();
        assert_eq!(fs::read(dir.path.join("dst/a copy 2.txt")).unwrap(), b"1");
    }

    #[test]
    fn paste_cut_removes_source_on_same_filesystem() {
        let dir = TestDir::new();
        fs::write(dir.path.join("m.txt"), b"move-me").unwrap();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "dst".to_string(),
            is_dir: true,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();

        let result = paste_paths(PastePathsRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            src_paths: vec!["m.txt".to_string()],
            dst_dir: "dst".to_string(),
            is_cut: true,
            runtime_os: SessionTerminalOs::Linux,
        });
        assert!(result.is_ok());
        assert!(!dir.path.join("m.txt").exists());
        assert_eq!(
            fs::read(dir.path.join("dst/m.txt")).unwrap(),
            b"move-me".to_vec()
        );
    }

    #[test]
    fn paste_refuses_into_itself() {
        let dir = TestDir::new();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "self_dir".to_string(),
            is_dir: true,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();

        let result = paste_paths(PastePathsRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            src_paths: vec!["self_dir".to_string()],
            dst_dir: "self_dir".to_string(),
            is_cut: false,
            runtime_os: SessionTerminalOs::Linux,
        });
        assert!(result.is_err());
    }

    #[test]
    fn rename_file_succeeds() {
        let dir = TestDir::new();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "old.txt".to_string(),
            is_dir: false,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();

        let result = rename_path(RenamePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            old_path: "old.txt".to_string(),
            new_path: "new.txt".to_string(),
            runtime_os: SessionTerminalOs::Linux,
        });
        assert!(result.is_ok());
        assert!(!dir.path.join("old.txt").exists());
        assert!(dir.path.join("new.txt").exists());
    }

    #[test]
    fn rename_refuses_when_destination_exists() {
        let dir = TestDir::new();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "src.txt".to_string(),
            is_dir: false,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "dst.txt".to_string(),
            is_dir: false,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();

        let result = rename_path(RenamePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            old_path: "src.txt".to_string(),
            new_path: "dst.txt".to_string(),
            runtime_os: SessionTerminalOs::Linux,
        });
        assert!(result.is_err());
        // Both source and destination untouched.
        assert!(dir.path.join("src.txt").exists());
        assert!(dir.path.join("dst.txt").exists());
    }

    #[test]
    fn rename_no_op_when_paths_match() {
        let dir = TestDir::new();
        create_path(CreatePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            path: "same.txt".to_string(),
            is_dir: false,
            runtime_os: SessionTerminalOs::Linux,
        })
        .unwrap();

        let result = rename_path(RenamePathRequest {
            base_path: Some(dir.path.to_string_lossy().into_owned()),
            old_path: "same.txt".to_string(),
            new_path: "same.txt".to_string(),
            runtime_os: SessionTerminalOs::Linux,
        });
        assert!(result.is_ok());
        assert!(dir.path.join("same.txt").exists());
    }
}
