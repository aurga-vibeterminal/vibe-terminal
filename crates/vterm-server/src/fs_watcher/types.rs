use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use notify::{event::ModifyKind, EventKind, RecursiveMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchMode {
    NonRecursive,
    Recursive,
}

impl From<WatchMode> for RecursiveMode {
    fn from(mode: WatchMode) -> Self {
        match mode {
            WatchMode::NonRecursive => RecursiveMode::NonRecursive,
            WatchMode::Recursive => RecursiveMode::Recursive,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchRequest {
    pub path: PathBuf,
    pub mode: WatchMode,
}

impl WatchRequest {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            mode: WatchMode::Recursive,
        }
    }

    pub fn with_mode(path: impl Into<PathBuf>, mode: WatchMode) -> Self {
        Self {
            path: path.into(),
            mode,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchedPath {
    pub path: PathBuf,
    pub mode: WatchMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileSystemEventKind {
    Created,
    Modified,
    Removed,
    Renamed,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileSystemNotification {
    pub watched_root: Option<PathBuf>,
    pub paths: Vec<PathBuf>,
    pub kind: FileSystemEventKind,
    pub received_at: SystemTime,
}

impl FileSystemNotification {
    pub(super) fn from_notify_event(
        event: notify::Event,
        watched_roots: &[PathBuf],
    ) -> FileSystemNotification {
        let watched_root = find_watched_root(&event.paths, watched_roots);
        FileSystemNotification {
            watched_root,
            kind: FileSystemEventKind::from(event.kind),
            paths: event.paths,
            received_at: SystemTime::now(),
        }
    }
}

impl From<EventKind> for FileSystemEventKind {
    fn from(kind: EventKind) -> Self {
        match kind {
            EventKind::Create(_) => FileSystemEventKind::Created,
            EventKind::Modify(ModifyKind::Name(_)) => FileSystemEventKind::Renamed,
            EventKind::Modify(_) => FileSystemEventKind::Modified,
            EventKind::Remove(_) => FileSystemEventKind::Removed,
            _ => FileSystemEventKind::Other,
        }
    }
}

fn find_watched_root(paths: &[PathBuf], watched_roots: &[PathBuf]) -> Option<PathBuf> {
    paths
        .iter()
        .filter_map(|path| {
            watched_roots
                .iter()
                .filter(|root| path_is_within(path, root))
                .max_by_key(|root| root.components().count())
        })
        .next()
        .cloned()
}

fn path_is_within(path: &Path, root: &Path) -> bool {
    path == root || path.starts_with(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::{event::CreateKind, Event};

    #[test]
    fn maps_notify_create_event() {
        let event = Event {
            kind: EventKind::Create(CreateKind::File),
            paths: vec![PathBuf::from("D:/repo/src/main.rs")],
            attrs: Default::default(),
        };

        let notification = FileSystemNotification::from_notify_event(
            event,
            &[PathBuf::from("D:/repo"), PathBuf::from("D:/repo/src")],
        );

        assert_eq!(notification.kind, FileSystemEventKind::Created);
        assert_eq!(
            notification.watched_root,
            Some(PathBuf::from("D:/repo/src"))
        );
    }
}
