#![allow(dead_code, unused_imports)]

mod error;
mod manager;
mod types;

pub use error::FsWatcherError;
pub(crate) use manager::send_message_to_session_viewers;
pub use manager::FsWatcherManager;
pub use types::{
    FileSystemEventKind, FileSystemNotification, WatchMode, WatchRequest, WatchedPath,
};
