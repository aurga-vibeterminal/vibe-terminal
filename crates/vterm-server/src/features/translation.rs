//! Translation of chat/AI messages via the configured translation LLM
//! addon.

use std::sync::Arc;

use tracing::warn;
use vterm_common::SessionId;
use vterm_core::protocol::Message;

use crate::addon::{resolve_llm_addon_id, run_chat_completion, AddonHost, LlmRequestContext};
use crate::config::TerminalServerConfig;
use crate::connection::Connection;

impl Connection {
    pub(crate) async fn handle_translation_message(
        &mut self,
        message: Message,
    ) -> anyhow::Result<()> {
        match message {
            Message::TranslateMessage {
                window_id,
                component_id,
                session_id,
                message_id,
                text,
                source_file,
            } => {
                if !self.ensure_session_access(session_id).await? {
                    return Ok(());
                }
                let result = self
                    .translate_message(
                        window_id,
                        component_id,
                        session_id,
                        message_id,
                        text,
                        source_file,
                    )
                    .await;
                self.send_message(result).await?;
            }
            other => {
                warn!("Unhandled translation message: {}", other.msg_name());
            }
        }

        Ok(())
    }

    /// Translate a chat message via the configured translation LLM addon.
    async fn translate_message(
        &self,
        window_id: u64,
        component_id: u64,
        session_id: SessionId,
        message_id: String,
        text: String,
        source_file: Option<String>,
    ) -> Message {
        let config = self.config_manager.snapshot().await;
        let addon_host = self.addon_host.clone();
        let message_id_clone = message_id.clone();
        let result = tokio::task::spawn_blocking(move || {
            translate_text_with_llm(&addon_host, &config, &text, source_file.as_deref())
        })
        .await;
        match result {
            Ok(translation_result) => Message::TranslateMessageResult {
                window_id,
                component_id,
                session_id,
                message_id: message_id_clone,
                translation: translation_result.clone().ok(),
                error: translation_result.err(),
            },
            Err(error) => {
                warn!("[translate] spawn blocking task failed: {}", error);
                Message::TranslateMessageResult {
                    window_id,
                    component_id,
                    session_id,
                    message_id: message_id_clone,
                    translation: None,
                    error: Some(error.to_string()),
                }
            }
        }
    }
}

/// Translate text using the configured translation LLM addon.
///
/// `source_file` is the file the text was selected from (editor / diff
/// view), given to the LLM as context so it can recognize the language's
/// comment syntax; `None` for prose sources like chat or commit messages.
fn translate_text_with_llm(
    addon_host: &Arc<AddonHost>,
    config: &TerminalServerConfig,
    text: &str,
    source_file: Option<&str>,
) -> Result<String, String> {
    let addon_id = resolve_llm_addon_id(config, "translation")
        .ok_or_else(|| "translation addon is not configured or disabled".to_string())?;

    let llm_config = &config.llm.translation;
    let output_language = if llm_config.output_language.is_empty() {
        "en".to_string()
    } else {
        llm_config.output_language.clone()
    };

    let system_prompt = build_translation_system_prompt(&output_language, source_file);

    let messages_json = serde_json::json!([
        {"role": "system", "content": system_prompt},
        {"role": "user", "content": text},
    ])
    .to_string();

    let stats_context = LlmRequestContext {
        category: "translation".to_string(),
    };

    let completion = run_chat_completion(
        addon_host,
        config,
        &addon_id,
        &messages_json,
        "{}",
        Some(&llm_config.provider.user_config),
        Some(std::time::Duration::from_secs(60)),
        Some(&stats_context),
    );

    if completion.success {
        completion
            .content
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .ok_or_else(|| "translation returned empty result".to_string())
    } else {
        Err(completion
            .error
            .unwrap_or_else(|| "translation failed".to_string()))
    }
}

/// Build the translation system prompt.
///
/// `source_file` switches on the source-code rules. The model is told the
/// file name AND — derived from its extension — exactly which comment
/// markers that language uses, so it never has to guess whether `///` or
/// `#` is content or decoration. The comment rule is deliberately explicit
/// and negative: with the earlier "strip the comment markers" phrasing,
/// models tended to keep the markers and translate around them (the
/// Markdown preservation rule pulled in the same direction), so a selected
/// `///` doc line or `//!` doc block came back with the markers still
/// prefixed. The rule now names the markers to remove, prohibits
/// re-emitting them, and shows a minimal shape example. Markdown formatting
/// that belongs to the comment's content (headings, lists, code fences)
/// stays intact — only the comment markers are removed.
fn build_translation_system_prompt(output_language: &str, source_file: Option<&str>) -> String {
    let mut prompt = format!(
        "You are a professional translator. Translate the following text to {output_language}. \
         Preserve the original meaning and tone. \
         If the text is Markdown prose (a chat message, commit message or document), keep the \
         Markdown formatting fully intact (headings, lists, tables, links, code fences, inline \
         code) and translate only the natural-language content."
    );
    if let Some(file) = source_file.filter(|f| !f.trim().is_empty()) {
        let syntax = comment_syntax_for(file);
        let example_input = format!(
            "{}Classify expected disconnect conditions from the WebSocket layer.{}",
            syntax.example_open, syntax.example_close
        );
        prompt.push_str(&format!(
            "\n\nThe text was selected from the source file `{file}`; comment markers in this \
             file's language: {markers}. Whenever the selected text contains comment markers — \
             even a single comment line — those markers are NOT part of the content: remove them \
             and translate the comment text as prose, keeping the original line breaks.{block_note} \
             Never re-emit comment markers: {prohibition}.\
             \nExample input:\n  {example_input}\
             \nCorrect output shape: one plain prose line in {output_language} (the translation \
             of \"Classify expected disconnect conditions from the WebSocket layer.\") with NO \
             comment marker — re-adding it at the start of the output is an error.\
             \nMarkdown formatting that belongs to the comment content (headings, list items \
             including * bullets, code fences, inline code) IS content: keep it intact.\
             \nIf the selection is ordinary code rather than comment text, translate only the \
             natural-language parts (comments, string literals) and keep the code, identifiers \
             and structure unchanged.",
            markers = syntax.markers,
            block_note = syntax.block_note,
            prohibition = syntax.prohibition,
        ));
    }
    prompt.push_str(" Output ONLY the translation, no explanations or notes.");
    prompt
}

/// Comment syntax of a source-file family, spelled out in the translation
/// prompt so the model never has to guess which markers decorate the text.
struct CommentSyntax {
    /// Marker list for the rule sentence ("//, ///, //! and /* ... */").
    markers: &'static str,
    /// Extra sentence about block-comment continuation markers, empty when
    /// the family has none.
    block_note: &'static str,
    /// Prefix/suffix wrapped around the shape example's sentence.
    example_open: &'static str,
    example_close: &'static str,
    /// Explicit prohibition on re-emitting the markers. Scoped to the
    /// family's own markers so e.g. `#` headings inside Rust doc comments
    /// (Markdown content) are not outlawed along with `///`.
    prohibition: &'static str,
}

const COMMENT_SYNTAX_SLASH: CommentSyntax = CommentSyntax {
    markers: "//, ///, //! and /* ... */",
    block_note: " Inside /* ... */ blocks, the decorative leading * on each continuation line \
                 is a marker too — drop it as well.",
    example_open: "/// ",
    example_close: "",
    prohibition: "no output line may start with //, /// or //!, and /* and */ must not appear",
};
const COMMENT_SYNTAX_HASH: CommentSyntax = CommentSyntax {
    markers: "#",
    block_note: "",
    example_open: "# ",
    example_close: "",
    prohibition: "no output line may start with #",
};
const COMMENT_SYNTAX_DASH: CommentSyntax = CommentSyntax {
    markers: "--",
    block_note: "",
    example_open: "-- ",
    example_close: "",
    prohibition: "no output line may start with --",
};
const COMMENT_SYNTAX_SEMICOLON: CommentSyntax = CommentSyntax {
    markers: ";",
    block_note: "",
    example_open: "; ",
    example_close: "",
    prohibition: "no output line may start with ;",
};
const COMMENT_SYNTAX_HTML: CommentSyntax = CommentSyntax {
    markers: "<!-- ... -->",
    block_note: "",
    example_open: "<!-- ",
    example_close: " -->",
    prohibition: "<!-- and --> must not appear",
};
const COMMENT_SYNTAX_PERCENT: CommentSyntax = CommentSyntax {
    markers: "%",
    block_note: "",
    example_open: "% ",
    example_close: "",
    prohibition: "no output line may start with %",
};
const COMMENT_SYNTAX_PAREN_STAR: CommentSyntax = CommentSyntax {
    markers: "(* ... *)",
    block_note: "",
    example_open: "(* ",
    example_close: " *)",
    prohibition: "(* and *) must not appear",
};
const COMMENT_SYNTAX_BANG: CommentSyntax = CommentSyntax {
    markers: "!",
    block_note: "",
    example_open: "! ",
    example_close: "",
    prohibition: "no output line may start with !",
};
const COMMENT_SYNTAX_GENERIC: CommentSyntax = CommentSyntax {
    markers: "//, ///, //!, /* ... */, #, --, ; or <!-- ... -->, depending on the language",
    block_note: " Inside /* ... */ blocks, the decorative leading * on each continuation line \
                 is a marker too — drop it as well.",
    example_open: "/// ",
    example_close: "",
    prohibition: "no output line may start with //, ///, //!, #, --, ; or *, and /*, */, <!-- \
                  and --> must not appear",
};

/// Map a source file to its comment syntax family by extension (the path
/// may use either separator and may be absolute).
fn comment_syntax_for(source_file: &str) -> CommentSyntax {
    let name = source_file
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(source_file)
        .to_ascii_lowercase();
    // Well-known extensionless / special files.
    if matches!(
        name.as_str(),
        "dockerfile" | "makefile" | "gnumakefile" | "cmakelists.txt" | "rakefile" | "gemfile"
    ) {
        return COMMENT_SYNTAX_HASH;
    }
    let Some((_, ext)) = name.rsplit_once('.') else {
        return COMMENT_SYNTAX_GENERIC;
    };
    match ext {
        "rs" | "js" | "jsx" | "mjs" | "cjs" | "ts" | "tsx" | "mts" | "cts" | "java" | "kt"
        | "kts" | "scala" | "sc" | "groovy" | "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hh"
        | "cs" | "go" | "swift" | "php" | "dart" | "m" | "mm" | "zig" | "v" | "sv" | "proto"
        | "thrift" | "scss" | "less" | "css" | "jsonc" | "json5" => COMMENT_SYNTAX_SLASH,
        "py" | "pyi" | "rb" | "sh" | "bash" | "zsh" | "fish" | "toml" | "yaml" | "yml" | "pl"
        | "pm" | "r" | "jl" | "nim" | "coffee" | "ex" | "exs" | "ps1" | "psm1" | "psd1" | "awk"
        | "tcl" | "ini" | "cfg" | "conf" => COMMENT_SYNTAX_HASH,
        "lua" | "sql" | "hs" | "lhs" | "elm" | "ada" | "adb" | "ads" | "vhd" | "vhdl"
        | "applescript" => COMMENT_SYNTAX_DASH,
        "lisp" | "lsp" | "cl" | "clj" | "cljs" | "cljc" | "scm" | "rkt" | "el" | "asm" | "s"
        | "nasm" => COMMENT_SYNTAX_SEMICOLON,
        "html" | "htm" | "xml" | "xhtml" | "svg" | "vue" | "svelte" | "md" | "markdown" | "mdx"
        | "plist" => COMMENT_SYNTAX_HTML,
        "erl" | "hrl" | "tex" | "sty" | "cls" | "pro" | "prolog" => COMMENT_SYNTAX_PERCENT,
        "ml" | "mli" | "pas" | "pp" => COMMENT_SYNTAX_PAREN_STAR,
        "f" | "f90" | "f95" | "for" => COMMENT_SYNTAX_BANG,
        _ => COMMENT_SYNTAX_GENERIC,
    }
}

#[cfg(test)]
mod tests {
    use super::build_translation_system_prompt;

    #[test]
    fn translation_prompt_for_prose_sources_skips_comment_rules() {
        let prompt = build_translation_system_prompt("zh-CN", None);
        assert!(prompt.contains("zh-CN"));
        assert!(prompt.contains("Markdown"));
        // No source file: the comment-marker rules stay out entirely.
        assert!(!prompt.contains("//!"));
        assert!(!prompt.contains("source file"));
        assert!(prompt.ends_with("no explanations or notes."));
    }

    #[test]
    fn translation_prompt_for_file_sources_demands_marker_free_output() {
        let prompt = build_translation_system_prompt("zh-CN", Some("src/latency.rs"));
        // The file name and its language's exact comment markers are given.
        assert!(prompt.contains("src/latency.rs"));
        assert!(prompt.contains("comment markers in this file's language: //, ///, //!"));
        // Single comment lines are covered too (the reported failure mode).
        assert!(prompt.contains("even a single comment line"));
        // The prohibition on re-emitting markers is explicit, and scoped to
        // the slash family (so `#` Markdown headings stay legal).
        assert!(prompt.contains("Never re-emit comment markers"));
        assert!(prompt.contains("no output line may start with //"));
        assert!(!prompt.contains("start with #"));
        // The shape example demonstrates the single-line doc-comment case.
        assert!(prompt.contains("/// Classify expected disconnect conditions"));
        // Markdown belonging to the comment's content stays intact.
        assert!(prompt.contains("IS content"));
        // Code selections keep their structure.
        assert!(prompt.contains("identifiers and structure unchanged"));
        assert!(prompt.ends_with("no explanations or notes."));
    }

    #[test]
    fn translation_prompt_uses_the_extensions_comment_syntax() {
        // Python: # is the marker, and it IS prohibited (no Markdown heading
        // conflict in this family). The /* */ continuation note is irrelevant
        // and stays out.
        let prompt = build_translation_system_prompt("zh-CN", Some("app/main.py"));
        assert!(prompt.contains("comment markers in this file's language: #"));
        assert!(prompt.contains("no output line may start with #"));
        assert!(prompt.contains("# Classify expected disconnect conditions"));
        assert!(!prompt.contains("decorative leading *"));

        // Rust: the /* */ continuation note applies.
        let prompt = build_translation_system_prompt("zh-CN", Some("src/lib.rs"));
        assert!(prompt.contains("decorative leading *"));

        // Lua: -- is the marker.
        let prompt = build_translation_system_prompt("en", Some("init.lua"));
        assert!(prompt.contains("comment markers in this file's language: --"));
        assert!(prompt.contains("-- Classify expected disconnect conditions"));

        // HTML: <!-- ... --> delimiters.
        let prompt = build_translation_system_prompt("en", Some("index.html"));
        assert!(prompt.contains("<!-- ... -->"));
        assert!(prompt.contains("<!-- Classify expected disconnect conditions"));

        // Unknown / extensionless files fall back to the generic list.
        let prompt = build_translation_system_prompt("en", Some("notes"));
        assert!(prompt.contains("depending on the language"));
    }

    #[test]
    fn translation_prompt_ignores_blank_source_file() {
        let prompt = build_translation_system_prompt("en", Some("   "));
        assert!(!prompt.contains("source file"));
    }
}
