use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use vterm_common::PathNameMatch;

const DEFAULT_MAX_RESULTS: usize = 1000;
/// Hard ceiling on entries examined so an absurdly large tree can't pin a
/// worker thread indefinitely.
const MAX_ENTRIES_SCANNED: usize = 200_000;

#[derive(Debug, Clone)]
pub(super) struct SearchPathNamesOptions {
    pub(super) root: PathBuf,
    pub(super) query: String,
    pub(super) max_results: usize,
}

/// Recursively walk `root`, emitting a [`PathNameMatch`] for every entry whose
/// name contains `query` (case-insensitive). Directories are matched *and*
/// descended into; files are only matched. File contents are never read, so
/// this is a cheap walk.
///
/// Honors `.gitignore`, `.ignore`, `.git/info/exclude`, and the global git
/// ignore file (via the `ignore` crate) — ignored paths are neither matched
/// nor descended. Hidden files are *not* excluded purely for being hidden
/// (only `.gitignore` membership matters), so e.g. an un-ignored `.env` stays
/// searchable.
///
/// `on_match` is called per match and `is_cancelled` is polled to allow early
/// termination (e.g. when a newer search supersedes this one).
pub(super) fn search_path_names_stream<F, C>(
    options: SearchPathNamesOptions,
    mut on_match: F,
    mut is_cancelled: C,
) -> Result<bool, String>
where
    F: FnMut(PathNameMatch) -> Result<(), String>,
    C: FnMut() -> bool,
{
    let query = options.query.trim();
    if query.is_empty() {
        return Ok(true);
    }
    if !options.root.is_dir() {
        return Err("Search root is not a directory".to_string());
    }

    let needle = query.to_lowercase();
    let max_results = if options.max_results == 0 {
        DEFAULT_MAX_RESULTS
    } else {
        options.max_results
    };

    // Standard filters enable .gitignore/.ignore/.git/info/exclude/global
    // gitignore. We deliberately leave hidden-file filtering OFF so only
    // gitignore membership (not dotfile-ness) decides visibility.
    let walker = WalkBuilder::new(&options.root)
        .standard_filters(true)
        .hidden(false)
        .filter_entry(|entry| entry.file_name().to_string_lossy() != ".git")
        .build();

    let mut match_count = 0usize;
    let mut entries_scanned = 0usize;
    let mut cancelled = false;

    for result in walker {
        if (is_cancelled)() {
            cancelled = true;
            break;
        }
        if match_count >= max_results || entries_scanned >= MAX_ENTRIES_SCANNED {
            break;
        }
        entries_scanned += 1;

        // The walker yields the root itself at depth 0; skip it — searching the
        // nailed folder's own name against the query is never useful.
        let entry = match result {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        if entry.depth() == 0 {
            continue;
        }

        let Some(file_type) = entry.file_type() else {
            continue;
        };
        // Skip symlinks to avoid cycles; only real files/dirs are searchable.
        if entry.path_is_symlink() {
            continue;
        }
        let is_dir = file_type.is_dir();
        // A plain file is a leaf; a directory is both a candidate match and a
        // subtree to descend (the walker handles descent for us).
        let name = entry.file_name().to_string_lossy().into_owned();

        if name.to_lowercase().contains(&needle) {
            match_count += 1;
            let path = match_path(&options.root, entry.path());
            (on_match)(PathNameMatch { path, name, is_dir })?;
        }
    }

    Ok(!cancelled)
}

/// Render the match's full path. We surface the entry's absolute path with
/// separators normalized to `/` so the client can build expand requests that
/// match what `request_path_content` returns regardless of host OS.
fn match_path(root: &Path, entry_path: &Path) -> String {
    let _ = root;
    entry_path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn finds_case_insensitive_name_matches_recursively() {
        let root = TestDir::new();
        root.write_dir("src");
        root.write_dir("src/components");
        root.write("src/main.rs", "");
        root.write("src/README.md", "");
        root.write("src/components/Header.vue", "");

        let mut results = Vec::new();
        let completed = search_path_names_stream(
            SearchPathNamesOptions {
                root: root.path.clone(),
                query: "readme".to_string(),
                max_results: 100,
            },
            |m| {
                results.push(m);
                Ok(())
            },
            || false,
        )
        .unwrap();

        assert!(completed);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "README.md");
        assert!(!results[0].is_dir);
    }

    #[test]
    fn matches_folders_and_files() {
        let root = TestDir::new();
        root.write_dir("my-app");
        root.write_dir("my-app/utils");
        root.write("my-app/utils/app-config.ts", "");
        root.write("other.txt", "");

        let mut results = Vec::new();
        search_path_names_stream(
            SearchPathNamesOptions {
                root: root.path.clone(),
                query: "app".to_string(),
                max_results: 100,
            },
            |m| {
                results.push(m);
                Ok(())
            },
            || false,
        )
        .unwrap();

        let names: Vec<_> = results.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"my-app"));
        assert!(names.contains(&"app-config.ts"));
    }

    #[test]
    fn respects_max_results_cap() {
        let root = TestDir::new();
        for i in 0..20 {
            root.write(&format!("match_{i:02}.txt"), "");
        }

        let mut count = 0;
        search_path_names_stream(
            SearchPathNamesOptions {
                root: root.path.clone(),
                query: "match".to_string(),
                max_results: 5,
            },
            |_| {
                count += 1;
                Ok(())
            },
            || false,
        )
        .unwrap();

        assert_eq!(count, 5);
    }

    #[test]
    fn empty_query_returns_without_matching() {
        let root = TestDir::new();
        root.write("anything.txt", "");

        let mut count = 0;
        let completed = search_path_names_stream(
            SearchPathNamesOptions {
                root: root.path.clone(),
                query: "   ".to_string(),
                max_results: 100,
            },
            |_| {
                count += 1;
                Ok(())
            },
            || false,
        )
        .unwrap();

        assert!(completed);
        assert_eq!(count, 0);
    }

    #[test]
    fn honors_gitignore_patterns() {
        let root = TestDir::new();
        // Make the temp dir a git repo so the `ignore` crate treats it as the
        // root for .gitignore discovery.
        root.write(".git/HEAD", ""); // presence of .git is enough
        root.write(".gitignore", "node_modules/\n*.log\nbuild/\n");
        root.write_dir("node_modules");
        root.write("node_modules/needle.js", "");
        root.write_dir("build");
        root.write("build/needle.out", "");
        root.write("needle.log", "");
        root.write("src/needle.rs", "");

        let mut results = Vec::new();
        search_path_names_stream(
            SearchPathNamesOptions {
                root: root.path.clone(),
                query: "needle".to_string(),
                max_results: 100,
            },
            |m| {
                results.push(m);
                Ok(())
            },
            || false,
        )
        .unwrap();

        // node_modules/, build/, and needle.log are gitignored → only
        // src/needle.rs remains.
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "needle.rs");
    }

    #[test]
    fn excludes_git_directory_even_when_hidden_files_are_searchable() {
        let root = TestDir::new();
        root.write(".git/HEAD", "");
        root.write(".git/needle-index", "");
        root.write(".git/objects/needle-object", "");
        root.write(".needle.conf", "");

        let mut results = Vec::new();
        search_path_names_stream(
            SearchPathNamesOptions {
                root: root.path.clone(),
                query: "needle".to_string(),
                max_results: 100,
            },
            |m| {
                results.push(m);
                Ok(())
            },
            || false,
        )
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, ".needle.conf");
        assert!(results.iter().all(|r| !r.path.contains("/.git/")));
    }

    #[test]
    fn honors_nested_gitignore_in_subdirectory() {
        let root = TestDir::new();
        root.write(".git/HEAD", "");
        root.write_dir("vendor");
        root.write_dir("vendor/dist");
        // Nested .gitignore inside vendor/ ignores dist/.
        root.write("vendor/.gitignore", "dist/\n");
        root.write("vendor/dist/needle.js", "");
        root.write("vendor/needle.js", "");

        let mut results = Vec::new();
        search_path_names_stream(
            SearchPathNamesOptions {
                root: root.path.clone(),
                query: "needle".to_string(),
                max_results: 100,
            },
            |m| {
                results.push(m);
                Ok(())
            },
            || false,
        )
        .unwrap();

        let names: Vec<_> = results.iter().map(|r| r.name.as_str()).collect();
        // vendor/needle.js matches; vendor/dist/needle.js is ignored by the
        // nested .gitignore and must not appear.
        assert!(names.contains(&"needle.js"));
        assert!(results.iter().all(|r| !r.path.contains("/dist/")));
    }

    #[test]
    fn does_not_exclude_unignored_hidden_files() {
        let root = TestDir::new();
        root.write(".git/HEAD", "");
        // .env is hidden but NOT gitignored → it should still be searchable.
        root.write(".env.local", "");
        root.write(".needle.conf", "");

        let mut results = Vec::new();
        search_path_names_stream(
            SearchPathNamesOptions {
                root: root.path.clone(),
                query: "needle".to_string(),
                max_results: 100,
            },
            |m| {
                results.push(m);
                Ok(())
            },
            || false,
        )
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, ".needle.conf");
    }

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new() -> Self {
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("vterm-search-names-test-{id}"));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn write(&self, relative: &str, content: &str) {
            let path = self.path.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, content).unwrap();
        }

        fn write_dir(&self, relative: &str) {
            fs::create_dir_all(self.path.join(relative)).unwrap();
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
