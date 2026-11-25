use std::fs;
use std::path::{Path, PathBuf};

use regex::RegexBuilder;
use vterm_common::{FileSearchMatch, FileSearchResult};

const DEFAULT_MAX_RESULTS: usize = 10_000;
const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_FILES_SCANNED: usize = 50_000;

#[derive(Debug, Clone)]
pub(super) struct FindFilesOptions {
    pub(super) root: PathBuf,
    pub(super) query: String,
    pub(super) include: String,
    pub(super) exclude: String,
    pub(super) case_sensitive: bool,
    pub(super) whole_word: bool,
    pub(super) use_regex: bool,
    pub(super) max_results: usize,
}

pub(super) fn find_files(options: FindFilesOptions) -> Result<Vec<FileSearchResult>, String> {
    let mut results = Vec::new();
    find_files_stream(
        options,
        |result| {
            results.push(result);
            Ok(())
        },
        || false,
    )?;
    Ok(results)
}

pub(super) fn find_files_stream<F, C>(
    options: FindFilesOptions,
    mut on_result: F,
    mut is_cancelled: C,
) -> Result<bool, String>
where
    F: FnMut(FileSearchResult) -> Result<(), String>,
    C: FnMut() -> bool,
{
    let normalized_query = normalize_newlines(&options.query);
    let query = normalized_query.trim();
    if query.is_empty() {
        return Ok(true);
    }
    if !options.root.is_dir() {
        return Err("Search root is not a directory".to_string());
    }

    let matcher = LineMatcher::new(
        query,
        options.case_sensitive,
        options.whole_word,
        options.use_regex,
    )?;
    let include_patterns = parse_patterns(&options.include);
    let exclude_patterns = parse_patterns(&options.exclude);
    let (search_roots, include_patterns) =
        search_roots_and_include_patterns(&options.root, include_patterns);
    let max_results = if options.max_results == 0 {
        DEFAULT_MAX_RESULTS
    } else {
        options.max_results
    };

    let mut state = SearchState {
        root: options.root,
        matcher,
        multiline_query: query.contains('\n'),
        include_patterns,
        exclude_patterns,
        max_results,
        files_scanned: 0,
        match_count: 0,
        cancelled: false,
        on_result: &mut on_result,
        is_cancelled: &mut is_cancelled,
    };
    for root in search_roots {
        if state.cancelled
            || state.match_count >= state.max_results
            || state.files_scanned >= MAX_FILES_SCANNED
        {
            break;
        }
        visit_dir(&root, &mut state)?;
    }
    Ok(!state.cancelled)
}

struct SearchState<'a> {
    root: PathBuf,
    matcher: LineMatcher,
    multiline_query: bool,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    max_results: usize,
    files_scanned: usize,
    match_count: usize,
    cancelled: bool,
    on_result: &'a mut dyn FnMut(FileSearchResult) -> Result<(), String>,
    is_cancelled: &'a mut dyn FnMut() -> bool,
}

enum LineMatcher {
    Regex(regex::Regex),
    Literal {
        needle: String,
        case_sensitive: bool,
        whole_word: bool,
    },
}

impl LineMatcher {
    fn new(
        query: &str,
        case_sensitive: bool,
        whole_word: bool,
        use_regex: bool,
    ) -> Result<Self, String> {
        if use_regex {
            let pattern = if whole_word {
                format!(r"\b(?:{})\b", query)
            } else {
                query.to_string()
            };
            let regex = RegexBuilder::new(&pattern)
                .case_insensitive(!case_sensitive)
                .build()
                .map_err(|error| format!("Invalid regex: {error}"))?;
            return Ok(Self::Regex(regex));
        }

        Ok(Self::Literal {
            needle: query.to_string(),
            case_sensitive,
            whole_word,
        })
    }

    fn is_match(&self, line: &str) -> bool {
        match self {
            Self::Regex(regex) => regex.is_match(line),
            Self::Literal {
                needle,
                case_sensitive,
                whole_word,
            } => {
                let haystack = if *case_sensitive {
                    line.to_string()
                } else {
                    line.to_lowercase()
                };
                let needle = if *case_sensitive {
                    needle.to_string()
                } else {
                    needle.to_lowercase()
                };
                if !*whole_word {
                    return haystack.contains(&needle);
                }
                literal_whole_word_match(&haystack, &needle)
            }
        }
    }

    fn find_ranges(&self, text: &str) -> Result<Vec<(usize, usize)>, String> {
        match self {
            Self::Regex(regex) => Ok(regex
                .find_iter(text)
                .map(|match_| (match_.start(), match_.end()))
                .collect()),
            Self::Literal {
                needle,
                case_sensitive,
                whole_word,
            } => {
                let pattern = if *whole_word {
                    format!(r"\b(?:{})\b", regex::escape(needle))
                } else {
                    regex::escape(needle)
                };
                let regex = RegexBuilder::new(&pattern)
                    .case_insensitive(!case_sensitive)
                    .build()
                    .map_err(|error| format!("Invalid literal matcher: {error}"))?;
                Ok(regex
                    .find_iter(text)
                    .map(|match_| (match_.start(), match_.end()))
                    .collect())
            }
        }
    }
}

fn visit_dir(dir: &Path, state: &mut SearchState) -> Result<(), String> {
    if (state.is_cancelled)() {
        state.cancelled = true;
        return Ok(());
    }
    if state.match_count >= state.max_results || state.files_scanned >= MAX_FILES_SCANNED {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|error| error.to_string())?;
    for entry in entries {
        if (state.is_cancelled)() {
            state.cancelled = true;
            break;
        }
        if state.match_count >= state.max_results || state.files_scanned >= MAX_FILES_SCANNED {
            break;
        }

        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            if should_skip_dir(&path, state) {
                continue;
            }
            visit_dir(&path, state)?;
        } else if file_type.is_file() {
            state.files_scanned += 1;
            if should_search_file(&path, state) {
                search_file(&path, state)?;
            }
        }
    }

    Ok(())
}

fn search_file(path: &Path, state: &mut SearchState) -> Result<(), String> {
    if (state.is_cancelled)() {
        state.cancelled = true;
        return Ok(());
    }
    if fs::metadata(path)
        .map(|metadata| metadata.len() > MAX_FILE_BYTES)
        .unwrap_or(true)
    {
        return Ok(());
    }

    let Ok(content) = fs::read_to_string(path) else {
        return Ok(());
    };

    let mut matches = Vec::new();
    if state.multiline_query {
        let searchable = normalize_newlines(&content);
        for (start, end) in state.matcher.find_ranges(&searchable)? {
            if state.match_count >= state.max_results {
                break;
            }
            matches.push(FileSearchMatch {
                line: line_number_at(&searchable, start),
                text: multiline_match_text(&searchable, start, end),
            });
            state.match_count += 1;
        }
    } else {
        for (line_idx, line) in content.lines().enumerate() {
            if state.match_count >= state.max_results {
                break;
            }
            if state.matcher.is_match(line) {
                matches.push(FileSearchMatch {
                    line: (line_idx + 1) as u32,
                    text: line.to_string(),
                });
                state.match_count += 1;
            }
        }
    }

    if matches.is_empty() {
        return Ok(());
    }

    let relative = relative_path(&state.root, path);
    let file = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| relative.clone());
    (state.on_result)(FileSearchResult {
        file,
        path: relative,
        matches,
    })?;
    Ok(())
}

fn should_skip_dir(path: &Path, state: &SearchState) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    if matches!(
        name,
        ".git" | ".hg" | ".svn" | "node_modules" | "target" | "dist" | "build" | ".next" | ".cache"
    ) {
        return true;
    }
    matches_any_pattern(path, &state.root, &state.exclude_patterns)
}

fn should_search_file(path: &Path, state: &SearchState) -> bool {
    if matches_any_pattern(path, &state.root, &state.exclude_patterns) {
        return false;
    }
    state.include_patterns.is_empty()
        || matches_any_pattern(path, &state.root, &state.include_patterns)
}

fn parse_patterns(patterns: &str) -> Vec<String> {
    patterns
        .split(',')
        .map(str::trim)
        .filter(|pattern| !pattern.is_empty())
        .map(|pattern| pattern.replace('\\', "/"))
        .collect()
}

fn search_roots_and_include_patterns(
    root: &Path,
    include_patterns: Vec<String>,
) -> (Vec<PathBuf>, Vec<String>) {
    let mut search_roots = Vec::new();
    let mut filters = Vec::new();

    for pattern in include_patterns {
        let path = Path::new(&pattern);
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };

        if candidate.is_dir() {
            push_unique_path(&mut search_roots, candidate);
        } else if candidate.is_file() {
            filters.push(relative_path(root, &candidate));
        } else {
            filters.push(pattern);
        }
    }

    if search_roots.is_empty() {
        search_roots.push(root.to_path_buf());
    }

    (search_roots, filters)
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

fn matches_any_pattern(path: &Path, root: &Path, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }

    let relative = relative_path(root, path).replace('\\', "/");
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();

    patterns.iter().any(|pattern| {
        wildcard_match(pattern, &relative)
            || (!pattern.contains('/') && wildcard_match(pattern, &file_name))
            || relative.starts_with(pattern.trim_end_matches('/'))
    })
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let pattern = pattern.as_bytes();
    let text = text.as_bytes();
    let (mut p, mut t) = (0usize, 0usize);
    let mut star = None;
    let mut match_after_star = 0usize;

    while t < text.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == text[t]) {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            match_after_star = t;
        } else if let Some(star_pos) = star {
            p = star_pos + 1;
            match_after_star += 1;
            t = match_after_star;
        } else {
            return false;
        }
    }

    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

fn literal_whole_word_match(haystack: &str, needle: &str) -> bool {
    for (start, _) in haystack.match_indices(needle) {
        let end = start + needle.len();
        let before = haystack[..start].chars().next_back();
        let after = haystack[end..].chars().next();
        if !is_word_char(before) && !is_word_char(after) {
            return true;
        }
    }
    false
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn line_number_at(text: &str, byte_index: usize) -> u32 {
    text[..byte_index]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count() as u32
        + 1
}

fn multiline_match_text(text: &str, start: usize, end: usize) -> String {
    text[start..end].replace('\n', "\\n")
}

fn is_word_char(ch: Option<char>) -> bool {
    ch.is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn finds_literal_matches_with_include_and_exclude_filters() {
        let root = TestDir::new();
        root.write("src/main.rs", "fn main() {\n    println!(\"needle\");\n}\n");
        root.write("src/skip.rs", "needle\n");
        root.write("README.md", "needle\n");

        let results = find_files(FindFilesOptions {
            root: root.path.clone(),
            query: "needle".to_string(),
            include: "*.rs".to_string(),
            exclude: "src/skip.rs".to_string(),
            case_sensitive: true,
            whole_word: false,
            use_regex: false,
            max_results: 100,
        })
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "src/main.rs");
        assert_eq!(results[0].matches[0].line, 2);
    }

    #[test]
    fn searches_all_files_when_include_is_absolute_root_path() {
        let root = TestDir::new();
        root.write("src/main.rs", "needle\n");

        let results = find_files(FindFilesOptions {
            root: root.path.clone(),
            query: "needle".to_string(),
            include: root.path.to_string_lossy().into_owned(),
            exclude: String::new(),
            case_sensitive: true,
            whole_word: false,
            use_regex: false,
            max_results: 100,
        })
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "src/main.rs");
    }

    #[test]
    fn scopes_search_when_include_is_absolute_directory_path() {
        let root = TestDir::new();
        root.write("src/main.rs", "needle\n");
        root.write("tests/main.rs", "needle\n");

        let results = find_files(FindFilesOptions {
            root: root.path.clone(),
            query: "needle".to_string(),
            include: root.path.join("src").to_string_lossy().into_owned(),
            exclude: String::new(),
            case_sensitive: true,
            whole_word: false,
            use_regex: false,
            max_results: 100,
        })
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "src/main.rs");
    }

    #[test]
    fn supports_regex_and_whole_word_matching() {
        let root = TestDir::new();
        root.write("notes.txt", "alpha\nalphabet\nbeta\n");

        let results = find_files(FindFilesOptions {
            root: root.path.clone(),
            query: "alpha".to_string(),
            include: String::new(),
            exclude: String::new(),
            case_sensitive: false,
            whole_word: true,
            use_regex: true,
            max_results: 100,
        })
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matches.len(), 1);
        assert_eq!(results[0].matches[0].line, 1);
    }

    #[test]
    fn finds_literal_multiline_matches() {
        let root = TestDir::new();
        root.write("notes.txt", "before\nfoo\nbar\nafter\n");

        let results = find_files(FindFilesOptions {
            root: root.path.clone(),
            query: "foo\nbar".to_string(),
            include: String::new(),
            exclude: String::new(),
            case_sensitive: true,
            whole_word: false,
            use_regex: false,
            max_results: 100,
        })
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "notes.txt");
        assert_eq!(results[0].matches.len(), 1);
        assert_eq!(results[0].matches[0].line, 2);
        assert_eq!(results[0].matches[0].text, "foo\\nbar");
    }

    #[test]
    fn finds_multiline_matches_across_crlf_files() {
        let root = TestDir::new();
        root.write("notes.txt", "before\r\nFoo\r\nBar\r\nafter\r\n");

        let results = find_files(FindFilesOptions {
            root: root.path.clone(),
            query: "foo\nbar".to_string(),
            include: String::new(),
            exclude: String::new(),
            case_sensitive: false,
            whole_word: false,
            use_regex: false,
            max_results: 100,
        })
        .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].matches.len(), 1);
        assert_eq!(results[0].matches[0].line, 2);
        assert_eq!(results[0].matches[0].text, "Foo\\nBar");
    }

    #[test]
    fn returns_invalid_regex_errors() {
        let root = TestDir::new();
        root.write("notes.txt", "alpha\n");

        let error = find_files(FindFilesOptions {
            root: root.path.clone(),
            query: "[".to_string(),
            include: String::new(),
            exclude: String::new(),
            case_sensitive: false,
            whole_word: false,
            use_regex: true,
            max_results: 100,
        })
        .unwrap_err();

        assert!(error.starts_with("Invalid regex:"));
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
            let path = std::env::temp_dir().join(format!("vterm-find-files-test-{id}"));
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
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
