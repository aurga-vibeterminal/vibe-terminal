use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum FsWatcherError {
    #[error("path does not exist: {0}")]
    PathNotFound(PathBuf),
    #[error("path is not a directory: {0}")]
    NotDirectory(PathBuf),
    #[error("path is already watched: {0}")]
    AlreadyWatched(PathBuf),
    #[error("path is not watched: {0}")]
    NotWatched(PathBuf),
    #[error("refusing to watch a filesystem root or system directory: {0}")]
    ForbiddenWatchRoot(PathBuf),
    #[error("file system watcher error: {}", watcher_error_message(.0))]
    Watcher(#[from] notify::Error),
    #[error("session error: {0}")]
    Session(String),
}

fn watcher_error_message(error: &notify::Error) -> String {
    let mut message = error.to_string();
    #[cfg(target_os = "linux")]
    if matches!(error.kind, notify::ErrorKind::MaxFilesWatch) {
        message.push_str(
            ". The Linux inotify watch limit (fs.inotify.max_user_watches) was reached. \
             Raise it with `sudo sysctl fs.inotify.max_user_watches=524288` and persist it by \
             adding `fs.inotify.max_user_watches=524288` to /etc/sysctl.d/99-inotify.conf",
        );
    }
    message
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    #[test]
    fn watch_limit_error_includes_sysctl_hint() {
        let error =
            super::watcher_error_message(&notify::Error::new(notify::ErrorKind::MaxFilesWatch));
        assert!(error.contains("fs.inotify.max_user_watches=524288"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn other_errors_have_no_sysctl_hint() {
        let error = super::watcher_error_message(&notify::Error::path_not_found());
        assert!(!error.contains("fs.inotify.max_user_watches"));
    }
}
