use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use tracing::{trace, warn};
use vterm_common::{FileListItem, SessionTerminalOs};

use super::session_error::SessionError;

#[derive(Debug, Clone)]
pub(super) struct ListingTarget {
    pub(super) real_path: String,
    pub(super) filesystem_path: Option<PathBuf>,
    pub(super) runtime_os: SessionTerminalOs,
}

/// Resolve the requested listing path into a real directory or a virtual root listing.
pub(super) fn resolve_listing_target(
    base_path: Option<&str>,
    requested_path: &str,
    runtime_os: SessionTerminalOs,
) -> Result<ListingTarget, String> {
    let fallback = root_listing_target(runtime_os);
    let trimmed = requested_path.trim();
    trace!(
        "[path_listing] resolving target: base_path={:?} requested_path={:?} trimmed={:?} runtime_os={:?}",
        base_path, requested_path, trimmed, runtime_os
    );

    // An explicit root request (empty, "/", "\\") always targets the virtual root
    // regardless of base_path — this is how Windows drive listings are triggered.
    if runtime_os == SessionTerminalOs::Windows && is_windows_root_request(trimmed) {
        trace!(
            "[path_listing] using Windows virtual root for explicit root request: requested_path={:?}",
            requested_path
        );
        return Ok(fallback);
    }

    let candidate = if trimmed.is_empty() {
        base_path
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(str::to_string)
    } else {
        Some(trimmed.to_string())
    };

    let Some(candidate) = candidate else {
        warn!(
            "[path_listing] falling back to root: no requested path and no usable base path; requested_path={:?} base_path={:?} fallback={:?}",
            requested_path, base_path, fallback.real_path
        );
        return Ok(fallback);
    };
    if candidate.starts_with('~') {
        trace!(
            "[path_listing] candidate starts with '~'; attempting Unix home expansion before resolution: candidate={:?} base_path={:?}",
            candidate, base_path
        );
    }

    let resolved = resolve_filesystem_listing_path(base_path, &candidate, runtime_os)?;
    let Some(filesystem_path) = resolved else {
        trace!(
            "[path_listing] using virtual root target: candidate={:?} runtime_os={:?}",
            candidate,
            runtime_os
        );
        return Ok(fallback);
    };
    let exists = filesystem_path.exists();
    let is_dir = filesystem_path.is_dir();
    trace!(
        "[path_listing] resolved filesystem path: candidate={:?} filesystem_path={:?} exists={} is_dir={}",
        candidate, filesystem_path, exists, is_dir
    );

    if !exists || !is_dir {
        warn!(
            "[path_listing] falling back to root: resolved path is not an existing directory; candidate={:?} filesystem_path={:?} exists={} is_dir={} fallback={:?}",
            candidate, filesystem_path, exists, is_dir, fallback.real_path
        );
        return Ok(fallback);
    }

    trace!(
        "[path_listing] resolved listing target: real_path={:?} runtime_os={:?}",
        filesystem_path,
        runtime_os
    );
    Ok(ListingTarget {
        real_path: filesystem_path.to_string_lossy().into_owned(),
        filesystem_path: Some(filesystem_path),
        runtime_os,
    })
}

/// Resolve a watcher refresh without the interactive fallback to the
/// filesystem root. `None` means the watched directory no longer exists.
pub(super) fn resolve_existing_listing_target(
    base_path: Option<&str>,
    requested_path: &str,
    runtime_os: SessionTerminalOs,
) -> Result<Option<ListingTarget>, String> {
    let trimmed = requested_path.trim();
    if runtime_os == SessionTerminalOs::Windows && is_windows_root_request(trimmed) {
        return Ok(Some(root_listing_target(runtime_os)));
    }
    let candidate = if trimmed.is_empty() {
        base_path
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(str::to_string)
    } else {
        Some(trimmed.to_string())
    };
    let Some(candidate) = candidate else {
        return Ok(None);
    };
    let Some(filesystem_path) = resolve_filesystem_listing_path(base_path, &candidate, runtime_os)?
    else {
        return Ok(None);
    };
    if !filesystem_path.is_dir() {
        return Ok(None);
    }
    Ok(Some(ListingTarget {
        real_path: filesystem_path.to_string_lossy().into_owned(),
        filesystem_path: Some(filesystem_path),
        runtime_os,
    }))
}

/// Build the virtual root target used when the requested path is invalid.
fn root_listing_target(runtime_os: SessionTerminalOs) -> ListingTarget {
    match runtime_os {
        SessionTerminalOs::Windows => ListingTarget {
            real_path: String::new(),
            filesystem_path: None,
            runtime_os,
        },
        SessionTerminalOs::Linux | SessionTerminalOs::Macos => ListingTarget {
            real_path: "/".to_string(),
            filesystem_path: Some(PathBuf::from("/")),
            runtime_os,
        },
    }
}

/// Resolve a requested path into a filesystem path, or `None` for the Windows drive root.
fn resolve_filesystem_listing_path(
    base_path: Option<&str>,
    requested_path: &str,
    runtime_os: SessionTerminalOs,
) -> Result<Option<PathBuf>, String> {
    resolve_filesystem_path(base_path, requested_path, runtime_os)
}

/// Resolve a requested path into a filesystem path.
pub(super) fn resolve_filesystem_path(
    base_path: Option<&str>,
    requested_path: &str,
    runtime_os: SessionTerminalOs,
) -> Result<Option<PathBuf>, String> {
    if runtime_os == SessionTerminalOs::Windows {
        if is_windows_root_request(requested_path) {
            return Ok(None);
        }
        if let Some(path) = windows_drive_root_path(requested_path) {
            return Ok(Some(PathBuf::from(path)));
        }
    }

    let requested_path = expand_home_path_for_os(requested_path, runtime_os)?;
    let requested = PathBuf::from(&requested_path);
    if requested.is_absolute() {
        return Ok(Some(requested));
    }

    match base_path {
        Some(path) if !path.trim().is_empty() => {
            let base_path = expand_home_path_for_os(path.trim(), runtime_os)?;
            Ok(Some(Path::new(&base_path).join(requested)))
        }
        _ => std::env::current_dir()
            .map(|cwd| Some(cwd.join(requested)))
            .map_err(|error| error.to_string()),
    }
}

fn expand_home_path_for_os(path: &str, runtime_os: SessionTerminalOs) -> Result<String, String> {
    if runtime_os == SessionTerminalOs::Windows {
        return Ok(path.to_string());
    }

    expand_unix_home_path(path)
}

fn expand_unix_home_path(path: &str) -> Result<String, String> {
    if path == "~" {
        return std::env::var("HOME")
            .map_err(|_| "HOME is not set; cannot expand '~' path".to_string());
    }

    let Some(rest) = path.strip_prefix("~/") else {
        return Ok(path.to_string());
    };

    let home = std::env::var("HOME")
        .map_err(|_| "HOME is not set; cannot expand '~/' path".to_string())?;
    Ok(Path::new(&home).join(rest).to_string_lossy().into_owned())
}

/// Detect whether the client requested the synthetic Windows drive root.
pub(super) fn is_windows_root_request(path: &str) -> bool {
    matches!(path.trim(), "" | "/" | "\\")
}

/// Normalize a Windows drive request like `C:` into `C:\`.
pub(super) fn windows_drive_root_path(path: &str) -> Option<String> {
    let trimmed = path.trim();
    let mut chars = trimmed.chars();
    let drive = chars.next()?;
    if !drive.is_ascii_alphabetic() || chars.next()? != ':' {
        return None;
    }
    let suffix = chars.as_str();
    if suffix.is_empty() || suffix == "\\" || suffix == "/" {
        return Some(format!("{}:\\", drive.to_ascii_uppercase()));
    }
    None
}

/// Read and sort directory items without waiting for Git subprocesses.
///
/// Interactive listings use this fast phase for their first response, then
/// annotate the same snapshot in the background. Watcher-driven callers can
/// still use [`read_path_content`] when they need one complete response.
pub(super) fn read_path_content_fast(
    target: &ListingTarget,
) -> Result<Vec<FileListItem>, SessionError> {
    let mut items = Vec::new();

    if target.runtime_os == SessionTerminalOs::Windows && target.filesystem_path.is_none() {
        return read_windows_root_content();
    }

    let Some(path) = target.filesystem_path.as_ref() else {
        return Ok(items);
    };
    let entries = std::fs::read_dir(path)
        .map_err(|error| SessionError::FileSystemError(error.to_string()))?;

    for entry in entries {
        let entry = entry.map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| SessionError::FileSystemError(error.to_string()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if file_type.is_dir() {
            items.push(FileListItem::directory(name));
        } else {
            items.push(FileListItem::file(name));
        }
    }

    items.sort_by(|left, right| {
        right
            .is_dir
            .is_some()
            .cmp(&left.is_dir.is_some())
            .then_with(|| {
                left.name
                    .to_ascii_lowercase()
                    .cmp(&right.name.to_ascii_lowercase())
            })
            .then_with(|| left.name.cmp(&right.name))
    });

    mark_git_directory_ignored(path, &mut items);

    Ok(items)
}

/// Add Git modified/ignored flags to an already-read directory snapshot.
/// Returns `false` outside a Git worktree so callers can avoid a redundant
/// second response.
pub(super) fn annotate_path_content(target: &ListingTarget, items: &mut [FileListItem]) -> bool {
    let Some(path) = target.filesystem_path.as_deref() else {
        return false;
    };
    let annotated = annotate_git_metadata(path, items);
    mark_git_directory_ignored(path, items);
    annotated
}

/// Read a fully annotated directory listing in one blocking operation.
pub(super) fn read_path_content(target: ListingTarget) -> Result<Vec<FileListItem>, SessionError> {
    let mut items = read_path_content_fast(&target)?;
    annotate_path_content(&target, &mut items);
    Ok(items)
}

/// Mark the `.git` directory (and everything inside it) as git-ignored so it
/// renders dimmed in the file list, mirroring how git itself treats it.
/// `git check-ignore` does not report `.git` (it is special-cased internally
/// rather than matched by gitignore rules), so we enforce it explicitly: any
/// item named `.git`, and every item when listing *inside* a `.git` folder,
/// is flagged as ignored.
fn mark_git_directory_ignored(dir: &Path, items: &mut [FileListItem]) {
    let inside_git = dir
        .components()
        .any(|component| component.as_os_str() == ".git");

    for item in items {
        if inside_git || item.name == ".git" {
            item.git_ignored = true;
        }
    }
}

fn annotate_git_metadata(dir: &Path, items: &mut [FileListItem]) -> bool {
    if items.is_empty() {
        return false;
    }

    let started = Instant::now();
    let Some(repo_root) = git_repo_root_for_path(dir) else {
        return false;
    };
    let changed_paths = git_changed_paths(&repo_root, dir).unwrap_or_default();
    let ignored_paths = git_ignored_item_names(&repo_root, dir, items).unwrap_or_default();

    let item_count = items.len();
    for item in items.iter_mut() {
        item.git_modified = item_is_git_modified(&repo_root, dir, item, &changed_paths);
        item.git_ignored = ignored_paths.contains(&item.name);
    }
    trace!(
        "[path_listing] git annotation: dir={:?} items={} changed={} ignored={} elapsed={:?}",
        dir,
        item_count,
        changed_paths.len(),
        ignored_paths.len(),
        started.elapsed()
    );
    true
}

// Git annotation is by far the slowest part of a listing (up to three
// subprocess spawns per directory), while its inputs — the repo root, the
// changed-path set, ignore rules — change on a much slower timescale than
// clients navigate directories (Files explorer, viewing-path sheet). Cache
// each piece briefly so rapid browsing doesn't pay the git cost on every
// tap. Staleness only delays the modified/ignored badges, never the listing
// itself, and the fs watcher still drives full reloads.
const REPO_ROOT_TTL: Duration = Duration::from_secs(30);
const ANNOTATION_TTL: Duration = Duration::from_secs(2);
/// Bound on cached directories; on overflow the whole map is dropped (entries
/// are cheap to recompute).
const CACHE_CAP: usize = 512;

struct TtlCache<V> {
    map: HashMap<PathBuf, (Instant, V)>,
    ttl: Duration,
}

impl<V: Clone> TtlCache<V> {
    fn new(ttl: Duration) -> Self {
        Self {
            map: HashMap::new(),
            ttl,
        }
    }

    fn get(&self, key: &Path) -> Option<V> {
        match self.map.get(key) {
            Some((fetched, value)) if fetched.elapsed() < self.ttl => Some(value.clone()),
            _ => None,
        }
    }

    fn insert(&mut self, key: PathBuf, value: V) {
        if self.map.len() >= CACHE_CAP {
            self.map.clear();
        }
        self.map.insert(key, (Instant::now(), value));
    }
}

static REPO_ROOT_CACHE: LazyLock<Mutex<TtlCache<Option<PathBuf>>>> =
    LazyLock::new(|| Mutex::new(TtlCache::new(REPO_ROOT_TTL)));
static CHANGED_PATHS_CACHE: LazyLock<Mutex<TtlCache<Option<HashSet<String>>>>> =
    LazyLock::new(|| Mutex::new(TtlCache::new(ANNOTATION_TTL)));
static IGNORED_NAMES_CACHE: LazyLock<Mutex<TtlCache<Option<HashSet<String>>>>> =
    LazyLock::new(|| Mutex::new(TtlCache::new(ANNOTATION_TTL)));

fn git_repo_root_for_path(path: &Path) -> Option<PathBuf> {
    if let Some(cached) = REPO_ROOT_CACHE.lock().ok().and_then(|c| c.get(path)) {
        return cached;
    }
    let root = git_repo_root_for_path_uncached(path);
    if let Ok(mut cache) = REPO_ROOT_CACHE.lock() {
        cache.insert(path.to_path_buf(), root.clone());
    }
    root
}

fn git_repo_root_for_path_uncached(path: &Path) -> Option<PathBuf> {
    let mut command = std::process::Command::new("git");
    command
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(path);
    vterm_common::command_no_window(&mut command);
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        None
    } else {
        Some(PathBuf::from(root))
    }
}

fn git_changed_paths(repo_root: &Path, dir: &Path) -> Option<HashSet<String>> {
    if let Some(cached) = CHANGED_PATHS_CACHE.lock().ok().and_then(|c| c.get(dir)) {
        return cached;
    }
    let changed = git_changed_paths_uncached(repo_root, dir);
    if let Ok(mut cache) = CHANGED_PATHS_CACHE.lock() {
        cache.insert(dir.to_path_buf(), changed.clone());
    }
    changed
}

fn git_changed_paths_uncached(repo_root: &Path, dir: &Path) -> Option<HashSet<String>> {
    let pathspec = git_relative_path(repo_root, dir).unwrap_or_else(|| ".".to_string());
    let mut command = std::process::Command::new("git");
    command
        .args(["status", "--porcelain=v1", "-z", "-uall", "--", &pathspec])
        .env("GIT_OPTIONAL_LOCKS", "0")
        .current_dir(repo_root);
    vterm_common::command_no_window(&mut command);
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }

    let mut changed = HashSet::new();
    let mut entries = output.stdout.split(|byte| *byte == 0);
    while let Some(entry) = entries.next() {
        if entry.is_empty() || entry.len() < 4 {
            continue;
        }
        let index_status = entry[0] as char;
        let worktree_status = entry[1] as char;
        let path = String::from_utf8_lossy(&entry[3..]).replace('\\', "/");
        if path.is_empty() {
            continue;
        }
        if matches!(index_status, 'R' | 'C') || matches!(worktree_status, 'R' | 'C') {
            let _old_path = entries.next();
        }
        changed.insert(path);
    }

    Some(changed)
}

fn git_ignored_item_names(
    repo_root: &Path,
    dir: &Path,
    items: &[FileListItem],
) -> Option<HashSet<String>> {
    // Keyed by dir only: the result depends on the queried item names, so an
    // item created within the TTL renders without the ignored badge until the
    // entry expires — acceptable for a badge.
    if let Some(cached) = IGNORED_NAMES_CACHE.lock().ok().and_then(|c| c.get(dir)) {
        return cached;
    }
    let ignored = git_ignored_item_names_uncached(repo_root, dir, items);
    if let Ok(mut cache) = IGNORED_NAMES_CACHE.lock() {
        cache.insert(dir.to_path_buf(), ignored.clone());
    }
    ignored
}

fn git_ignored_item_names_uncached(
    repo_root: &Path,
    dir: &Path,
    items: &[FileListItem],
) -> Option<HashSet<String>> {
    let mut command = std::process::Command::new("git");
    command
        .args(["check-ignore", "-z", "--stdin", "--"])
        .current_dir(repo_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    vterm_common::command_no_window(&mut command);
    let mut child = command.spawn().ok()?;

    {
        let mut stdin = child.stdin.take()?;
        for item in items {
            // Query the plain path only. Directory-only patterns (e.g.
            // `node_modules/`) still match real on-disk directories because
            // git stats the path; listed items always exist. A synthetic
            // trailing-slash variant is not only redundant — with CRLF
            // .gitignore files, git ≤2.45 spuriously matches it against
            // blank lines (parsed as an empty pattern) for any path missing
            // from the index, wrongly flagging every untracked directory.
            let rel = git_relative_path(repo_root, &dir.join(&item.name))?;
            let _ = stdin.write_all(rel.as_bytes());
            let _ = stdin.write_all(&[0]);
        }
    }

    let output = child.wait_with_output().ok()?;
    if !output.status.success() && output.status.code() != Some(1) {
        return None;
    }

    let mut ignored = HashSet::new();
    for raw in output.stdout.split(|byte| *byte == 0) {
        if raw.is_empty() {
            continue;
        }
        let rel = String::from_utf8_lossy(raw)
            .trim_end_matches('/')
            .to_string();
        if let Some(name) = name_for_relative_child(repo_root, dir, &rel) {
            ignored.insert(name);
        }
    }
    Some(ignored)
}

fn item_is_git_modified(
    repo_root: &Path,
    dir: &Path,
    item: &FileListItem,
    changed_paths: &HashSet<String>,
) -> bool {
    let Some(rel) = git_relative_path(repo_root, &dir.join(&item.name)) else {
        return false;
    };
    if item.is_dir == Some(true) {
        let prefix = format!("{}/", rel.trim_end_matches('/'));
        changed_paths
            .iter()
            .any(|path| path == &rel || path.starts_with(&prefix))
    } else {
        changed_paths.contains(&rel)
    }
}

fn git_relative_path(repo_root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(repo_root).ok()?;
    let text = relative.to_string_lossy().replace('\\', "/");
    if text.is_empty() {
        Some(".".to_string())
    } else {
        Some(text)
    }
}

fn name_for_relative_child(repo_root: &Path, dir: &Path, rel: &str) -> Option<String> {
    let dir_rel = git_relative_path(repo_root, dir)?;
    if dir_rel == "." {
        return rel
            .split('/')
            .next()
            .filter(|name| !name.is_empty())
            .map(ToString::to_string);
    }
    let prefix = format!("{}/", dir_rel.trim_end_matches('/'));
    rel.strip_prefix(&prefix)
        .and_then(|rest| rest.split('/').next())
        .filter(|name| !name.is_empty())
        .map(ToString::to_string)
}

/// List Windows drive roots like `C:` and `D:` for the synthetic root view.
#[cfg(target_os = "windows")]
#[link(name = "kernel32")]
extern "system" {
    fn GetLogicalDrives() -> u32;
}

/// List Windows drive roots like `C:` and `D:` for the synthetic root view.
///
/// Drive enumeration is a filesystem operation and works fine on headless
/// Windows, so it deliberately avoids the `ui`-gated `windows-sys` dependency.
fn read_windows_root_content() -> Result<Vec<FileListItem>, SessionError> {
    #[cfg(target_os = "windows")]
    {
        let mask = unsafe { GetLogicalDrives() };
        if mask == 0 {
            return Err(SessionError::FileSystemError(
                "Unable to enumerate Windows drives".to_string(),
            ));
        }

        let mut items = Vec::new();
        for drive_index in 0..26 {
            if (mask & (1 << drive_index)) != 0 {
                let drive = char::from(b'A' + drive_index as u8);
                items.push(FileListItem::directory(format!("{drive}:")));
            }
        }
        return Ok(items);
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttl_cache_returns_inserted_value_within_ttl() {
        let mut cache = TtlCache::new(Duration::from_secs(60));
        cache.insert(PathBuf::from("/a"), Some(7));
        assert_eq!(cache.get(Path::new("/a")), Some(Some(7)));
        assert_eq!(cache.get(Path::new("/b")), None);
    }

    #[test]
    fn test_ttl_cache_expires_entries() {
        let mut cache = TtlCache::new(Duration::ZERO);
        cache.insert(PathBuf::from("/a"), Some(7));
        assert_eq!(cache.get(Path::new("/a")), None);
    }

    #[test]
    fn test_ttl_cache_cap_drops_entries_on_overflow() {
        let mut cache = TtlCache::new(Duration::from_secs(60));
        for index in 0..CACHE_CAP {
            cache.insert(PathBuf::from(format!("/dir{index}")), Some(index));
        }
        assert_eq!(cache.map.len(), CACHE_CAP);
        cache.insert(PathBuf::from("/overflow"), Some(0));
        assert_eq!(cache.map.len(), 1);
        assert_eq!(cache.get(Path::new("/dir0")), None);
        assert_eq!(cache.get(Path::new("/overflow")), Some(Some(0)));
    }

    #[test]
    fn test_resolve_filesystem_path_expands_unix_home_request() {
        let Some(home) = std::env::var("HOME").ok().filter(|home| !home.is_empty()) else {
            return;
        };

        let resolved =
            resolve_filesystem_path(None, "~/Documents", SessionTerminalOs::Linux).unwrap();

        assert_eq!(resolved, Some(Path::new(&home).join("Documents")));
    }

    #[test]
    fn test_resolve_filesystem_path_expands_unix_home_base() {
        let Some(home) = std::env::var("HOME").ok().filter(|home| !home.is_empty()) else {
            return;
        };

        let resolved =
            resolve_filesystem_path(Some("~/Documents"), "project", SessionTerminalOs::Linux)
                .unwrap();

        assert_eq!(resolved, Some(Path::new(&home).join("Documents/project")));
    }
}
