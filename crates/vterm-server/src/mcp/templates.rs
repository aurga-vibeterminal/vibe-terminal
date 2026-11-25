//! Per-app MCP setup templates for shell commands and conversation prompts.
//!
//! User entries in `McpConfig.templates` override a builtin by id or define
//! new apps. See docs/mcp.md ("Per-app templates").

use crate::config::{McpConfig, McpTemplateConfig};
use vterm_common::McpCommandTarget;

const TOKEN_BLOCK_OPEN: &str = "{#token}";
const TOKEN_BLOCK_CLOSE: &str = "{/token}";

const CLAUDE_STYLE_TEMPLATE: &str =
    "{executable} mcp add --transport http vterm-mcp {url}{#token} --header \"Authorization: Bearer {token}\"{/token}";
/// Codex reads the bearer token from an environment variable named by
/// `--bearer-token-env-var` instead of an inline flag. The variable is
/// `VTERM_MCP_TOKEN`, carried by the server-managed "Vibe Terminal MCP
/// Bearer Token" environment profile — users apply that profile to the
/// terminal before running the command (see CODEX_DESCRIPTION).
const CODEX_TEMPLATE: &str =
    "{executable} mcp add vterm-mcp --url {url}{#token} --bearer-token-env-var VTERM_MCP_TOKEN{/token}";
/// Kimi Code has no `mcp add` CLI; MCP servers are registered through the
/// agent conversation. The template is the chat prompt to send. Per
/// kimi.com/code/docs the HTTP server config takes a `bearerTokenEnvVar`
/// naming an environment variable, so the prompt passes `VTERM_MCP_TOKEN`
/// (the server-managed "Vibe Terminal MCP Bearer Token" profile) instead
/// of pasting the raw token into the chat.
const KIMI_TEMPLATE: &str =
    "/mcp-config add an HTTP MCP, url is {url}{#token}, bearerTokenEnvVar is VTERM_MCP_TOKEN{/token}";
/// Pi, OpenCode, Qwen Code, DeepSeek Harness, and Generic register MCP servers
/// through the agent conversation. The prompt asks the running agent to update
/// its own configuration directly and uses VTERM_MCP_TOKEN from the
/// server-managed "Vibe Terminal MCP Bearer Token" environment profile instead
/// of exposing the token in the chat.
const AGENT_CONVERSATION_TEMPLATE: &str =
    "Add an HTTP MCP server named vterm-mcp with URL {url}.{#token} Authenticate with the Bearer token from the VTERM_MCP_TOKEN environment variable.{/token} Update your MCP configuration directly.";

const KIMI_DESCRIPTION: &str = "Kimi Code has no MCP add command — this text is a chat prompt, \
     not a shell command. Apply the 'Vibe Terminal MCP Bearer Token' environment profile to the \
     terminal BEFORE launching Kimi (a running process won't see new variables); the prompt \
     registers the server with bearerTokenEnvVar=VTERM_MCP_TOKEN, so the token itself never \
     appears in the chat.";
const PI_DESCRIPTION: &str = "This text is a chat prompt, not a shell command. Apply the \
     'Vibe Terminal MCP Bearer Token' environment profile to the terminal BEFORE launching Pi, \
     then send the prompt in the Pi session so the agent can update its MCP configuration \
     directly. Run /mcp in the session to inspect the connection.";
const OPENCODE_DESCRIPTION: &str = "This text is a chat prompt, not a shell command. Apply the \
     'Vibe Terminal MCP Bearer Token' environment profile to the terminal BEFORE launching \
     OpenCode, then send the prompt in the OpenCode session so the agent can update its MCP \
     configuration directly.";
/// Qwen Code has no `mcp add` CLI either — its MCP servers live in
/// `~/.qwen/settings.json`, which the running agent can edit itself.
const QWEN_DESCRIPTION: &str = "This text is a chat prompt, not a shell command. Apply the \
     'Vibe Terminal MCP Bearer Token' environment profile to the terminal BEFORE launching \
     Qwen Code, then send the prompt in the Qwen Code session so the agent can update its MCP \
     configuration directly. Run /mcp in the session to inspect the connection.";
const DEEPSEEK_DESCRIPTION: &str = "This text is a chat prompt, not a shell command. Apply the \
     'Vibe Terminal MCP Bearer Token' environment profile to the terminal BEFORE launching \
     DeepSeek Harness, then send the prompt in the DeepSeek Harness session so the agent can \
     update its MCP configuration directly.";
const GENERIC_DESCRIPTION: &str = "This text is a chat prompt, not a shell command. Apply the \
     'Vibe Terminal MCP Bearer Token' environment profile to the terminal BEFORE launching your \
     agent, then send the prompt in its conversation so the agent can update its MCP \
     configuration directly.";
const CODEX_DESCRIPTION: &str = "Apply the 'Vibe Terminal MCP Bearer Token' environment \
     profile to the terminal before running this command — Codex reads the access token from \
     the VTERM_MCP_TOKEN environment variable it defines.";

struct BuiltinMcpTemplate {
    id: &'static str,
    display_name: &'static str,
    executable: &'static str,
    template: &'static str,
    description: &'static str,
}

impl BuiltinMcpTemplate {
    fn to_config(&self) -> McpTemplateConfig {
        McpTemplateConfig {
            display_name: self.display_name.to_string(),
            executable: self.executable.to_string(),
            template: self.template.to_string(),
            description: self.description.to_string(),
        }
    }
}

/// Keep in sync with the known-app list in
/// `session/terminal_observer/manager.rs`. The `default` entry is the
/// fallback for unknown executables and the starting point for new user apps.
const BUILTIN_MCP_TEMPLATES: &[BuiltinMcpTemplate] = &[
    BuiltinMcpTemplate {
        id: "claude",
        display_name: "Claude Code",
        executable: "claude",
        template: CLAUDE_STYLE_TEMPLATE,
        description: "",
    },
    BuiltinMcpTemplate {
        id: "codex",
        display_name: "Codex",
        executable: "codex",
        template: CODEX_TEMPLATE,
        description: CODEX_DESCRIPTION,
    },
    BuiltinMcpTemplate {
        id: "kimi",
        display_name: "Kimi Code",
        executable: "kimi",
        template: KIMI_TEMPLATE,
        description: KIMI_DESCRIPTION,
    },
    BuiltinMcpTemplate {
        id: "pi",
        display_name: "Pi Agent",
        executable: "pi",
        template: AGENT_CONVERSATION_TEMPLATE,
        description: PI_DESCRIPTION,
    },
    BuiltinMcpTemplate {
        id: "opencode",
        display_name: "OpenCode",
        executable: "opencode",
        template: AGENT_CONVERSATION_TEMPLATE,
        description: OPENCODE_DESCRIPTION,
    },
    BuiltinMcpTemplate {
        id: "qwen",
        display_name: "Qwen Code",
        executable: "qwen",
        template: AGENT_CONVERSATION_TEMPLATE,
        description: QWEN_DESCRIPTION,
    },
    BuiltinMcpTemplate {
        id: "deepseek",
        display_name: "DeepSeek Harness",
        executable: "dsh",
        template: AGENT_CONVERSATION_TEMPLATE,
        description: DEEPSEEK_DESCRIPTION,
    },
    BuiltinMcpTemplate {
        id: "default",
        display_name: "Generic",
        executable: "",
        template: AGENT_CONVERSATION_TEMPLATE,
        description: GENERIC_DESCRIPTION,
    },
];

/// Render a setup template: substitute `{executable}`, `{url}`, `{token}`
/// and include `{#token}...{/token}` blocks only when `token` is non-empty.
pub(crate) fn render_template(template: &str, executable: &str, url: &str, token: &str) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find(TOKEN_BLOCK_OPEN) {
        out.push_str(&rest[..start]);
        let after_open = &rest[start + TOKEN_BLOCK_OPEN.len()..];
        match after_open.find(TOKEN_BLOCK_CLOSE) {
            Some(end) => {
                if !token.is_empty() {
                    out.push_str(&after_open[..end]);
                }
                rest = &after_open[end + TOKEN_BLOCK_CLOSE.len()..];
            }
            None => {
                // Unclosed block: emit the remainder verbatim.
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out.replace("{executable}", executable)
        .replace("{url}", url)
        .replace("{token}", token)
        .trim()
        .to_string()
}

/// One entry of the merged builtin/user template view.
pub(crate) struct McpTemplateView {
    pub id: String,
    pub config: McpTemplateConfig,
    pub builtin: bool,
    /// True when a user entry exists for this id (overriding the builtin, or
    /// defining a non-builtin app).
    pub modified: bool,
}

/// Builtins in table order (each shadowed by a same-id user entry), followed
/// by non-builtin user entries sorted by id.
pub(crate) fn merged_templates(config: &McpConfig) -> Vec<McpTemplateView> {
    let mut views = Vec::new();
    for builtin in BUILTIN_MCP_TEMPLATES {
        match config.templates.get(builtin.id) {
            Some(user) => views.push(McpTemplateView {
                id: builtin.id.to_string(),
                config: user.clone(),
                builtin: true,
                modified: true,
            }),
            None => views.push(McpTemplateView {
                id: builtin.id.to_string(),
                config: builtin.to_config(),
                builtin: true,
                modified: false,
            }),
        }
    }
    let mut extra: Vec<(&String, &McpTemplateConfig)> = config
        .templates
        .iter()
        .filter(|(id, _)| !BUILTIN_MCP_TEMPLATES.iter().any(|b| b.id == id.as_str()))
        .collect();
    extra.sort_by(|a, b| a.0.cmp(b.0));
    for (id, user) in extra {
        views.push(McpTemplateView {
            id: id.clone(),
            config: user.clone(),
            builtin: false,
            modified: true,
        });
    }
    views
}

/// Resolve the template for an executable: match the `executable` field
/// first, then the id, else the `default` view.
pub(crate) fn find_template<'a>(
    views: &'a [McpTemplateView],
    executable: &str,
) -> &'a McpTemplateView {
    views
        .iter()
        .find(|view| !view.config.executable.is_empty() && view.config.executable == executable)
        .or_else(|| views.iter().find(|view| view.id == executable))
        .or_else(|| views.iter().find(|view| view.id == "default"))
        .expect("the default MCP template is always present")
}

/// Where a template's setup line must be typed: a template that begins with
/// `{executable}` renders a shell command for a plain terminal (the default);
/// anything else is a chat prompt that must be typed while that app is
/// running. Mirrors `McpCommandTarget` in vterm-common.
pub(crate) fn command_target(view: &McpTemplateView) -> McpCommandTarget {
    if view
        .config
        .template
        .trim_start()
        .starts_with("{executable}")
    {
        McpCommandTarget::Terminal
    } else {
        McpCommandTarget::InApp
    }
}

/// Render the effective setup line for a view. `fallback_executable` is used
/// when the template leaves `executable` empty (the generic template), so
/// unknown apps still get their own name in the command. The URL and token
/// come from the server MCP config, never from the template.
pub(crate) fn render_for_view(
    view: &McpTemplateView,
    fallback_executable: &str,
    config: &McpConfig,
) -> String {
    let executable = if view.config.executable.is_empty() {
        fallback_executable
    } else {
        view.config.executable.as_str()
    };
    let url = super::endpoint_url(config);
    render_template(
        &view.config.template,
        executable,
        &url,
        &config.access_token,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(template: &str, token: &str) -> String {
        render_template(
            template,
            "myapp",
            "http://localhost:7880/terminal/mcp",
            token,
        )
    }

    #[test]
    fn substitutes_placeholders() {
        assert_eq!(
            render("{executable} add {url} token={token}", "abc"),
            "myapp add http://localhost:7880/terminal/mcp token=abc"
        );
    }

    #[test]
    fn token_block_included_when_token_present() {
        assert_eq!(
            render(CLAUDE_STYLE_TEMPLATE, "abc"),
            "myapp mcp add --transport http vterm-mcp http://localhost:7880/terminal/mcp --header \"Authorization: Bearer abc\""
        );
    }

    #[test]
    fn token_block_dropped_when_token_empty() {
        let output = render(CLAUDE_STYLE_TEMPLATE, "");
        assert_eq!(
            output,
            "myapp mcp add --transport http vterm-mcp http://localhost:7880/terminal/mcp"
        );
        assert!(!output.contains("Bearer"));
    }

    #[test]
    fn codex_block_dropped_when_token_empty() {
        assert_eq!(
            render(CODEX_TEMPLATE, ""),
            "myapp mcp add vterm-mcp --url http://localhost:7880/terminal/mcp"
        );
    }

    /// Codex takes the token from the VTERM_MCP_TOKEN environment variable
    /// (the managed "Vibe Terminal MCP Bearer Token" profile); the command
    /// itself never inlines the secret.
    #[test]
    fn codex_command_never_inlines_the_token() {
        assert_eq!(
            render(CODEX_TEMPLATE, "s3cr3t"),
            "myapp mcp add vterm-mcp --url http://localhost:7880/terminal/mcp --bearer-token-env-var VTERM_MCP_TOKEN"
        );
    }

    #[test]
    fn merged_templates_returns_builtins_unmodified() {
        let views = merged_templates(&McpConfig::default());
        assert_eq!(views.len(), BUILTIN_MCP_TEMPLATES.len());
        let claude = views.iter().find(|v| v.id == "claude").unwrap();
        assert!(claude.builtin);
        assert!(!claude.modified);
        assert_eq!(claude.config.display_name, "Claude Code");
    }

    /// Apps whose setup flow needs more than the command line (conversation-
    /// style, or an env-var prerequisite) carry setup notes so the webui can
    /// explain the flow; plain-CLI apps ship without one.
    #[test]
    fn builtin_apps_with_extra_flow_are_described() {
        let views = merged_templates(&McpConfig::default());

        let kimi = views.iter().find(|v| v.id == "kimi").unwrap();
        assert!(kimi.config.template.starts_with("/mcp-config "));
        assert!(!kimi.config.description.is_empty());

        for id in ["pi", "opencode", "qwen", "deepseek", "default"] {
            let app = views.iter().find(|v| v.id == id).unwrap();
            assert_eq!(app.config.template, AGENT_CONVERSATION_TEMPLATE);
            assert!(!app.config.description.is_empty());
        }
        let qwen = views.iter().find(|v| v.id == "qwen").unwrap();
        assert_eq!(qwen.config.executable, "qwen");
        assert_eq!(qwen.config.display_name, "Qwen Code");
        let deepseek = views.iter().find(|v| v.id == "deepseek").unwrap();
        assert_eq!(deepseek.config.executable, "dsh");
        assert_eq!(deepseek.config.display_name, "DeepSeek Harness");

        let codex = views.iter().find(|v| v.id == "codex").unwrap();
        assert!(codex
            .config
            .template
            .contains("--bearer-token-env-var VTERM_MCP_TOKEN"));
        assert!(!codex.config.description.is_empty());

        let claude = views.iter().find(|v| v.id == "claude").unwrap();
        assert_eq!(claude.config.description, "");
    }

    /// Pi, OpenCode, DeepSeek Harness, and Generic take the token from the
    /// VTERM_MCP_TOKEN environment variable (the managed "Vibe Terminal MCP
    /// Bearer Token" profile); the conversation prompt never inlines the secret.
    #[test]
    fn agent_conversation_prompt_never_inlines_the_token() {
        assert_eq!(
            render(AGENT_CONVERSATION_TEMPLATE, "s3cr3t"),
            "Add an HTTP MCP server named vterm-mcp with URL http://localhost:7880/terminal/mcp. Authenticate with the Bearer token from the VTERM_MCP_TOKEN environment variable. Update your MCP configuration directly."
        );
    }

    /// Kimi's chat prompt passes the env-var name via `bearerTokenEnvVar`
    /// (kimi.com/code/docs); the raw token never appears in the conversation.
    #[test]
    fn kimi_prompt_never_inlines_the_token() {
        assert_eq!(
            render(KIMI_TEMPLATE, "s3cr3t"),
            "/mcp-config add an HTTP MCP, url is http://localhost:7880/terminal/mcp, bearerTokenEnvVar is VTERM_MCP_TOKEN"
        );
    }

    #[test]
    fn user_entry_overrides_builtin_and_marks_modified() {
        let mut config = McpConfig::default();
        let mut custom = McpTemplateConfig {
            display_name: "My Claude".to_string(),
            template: "custom {url}".to_string(),
            ..McpTemplateConfig::default()
        };
        custom.executable = "claude".to_string();
        config.templates.insert("claude".to_string(), custom);
        let views = merged_templates(&config);
        let claude = views.iter().find(|v| v.id == "claude").unwrap();
        assert!(claude.builtin);
        assert!(claude.modified);
        assert_eq!(claude.config.display_name, "My Claude");
        // Builtin order is preserved: claude still comes before codex.
        let ids: Vec<&str> = views.iter().map(|v| v.id.as_str()).collect();
        assert!(
            ids.iter().position(|id| *id == "claude").unwrap()
                < ids.iter().position(|id| *id == "codex").unwrap()
        );
    }

    #[test]
    fn non_builtin_user_entries_sorted_after_builtins() {
        let mut config = McpConfig::default();
        for id in ["zeta", "alpha"] {
            config.templates.insert(
                id.to_string(),
                McpTemplateConfig {
                    display_name: id.to_string(),
                    executable: id.to_string(),
                    template: "{executable} {url}".to_string(),
                    ..McpTemplateConfig::default()
                },
            );
        }
        let views = merged_templates(&config);
        let tail: Vec<&str> = views[views.len() - 2..]
            .iter()
            .map(|v| v.id.as_str())
            .collect();
        assert_eq!(tail, ["alpha", "zeta"]);
        assert!(!views[views.len() - 2].builtin);
        assert!(views[views.len() - 2].modified);
    }

    #[test]
    fn find_template_matches_executable_then_id_then_default() {
        let views = merged_templates(&McpConfig::default());
        assert_eq!(find_template(&views, "codex").id, "codex");
        assert_eq!(find_template(&views, "dsh").id, "deepseek");
        assert_eq!(find_template(&views, "unknown-cli").id, "default");

        // A user app whose id differs from its executable matches by
        // executable first.
        let mut config = McpConfig::default();
        config.templates.insert(
            "my-app".to_string(),
            McpTemplateConfig {
                display_name: "My App".to_string(),
                executable: "my-cli".to_string(),
                template: "{executable} {url}".to_string(),
                ..McpTemplateConfig::default()
            },
        );
        let views = merged_templates(&config);
        assert_eq!(find_template(&views, "my-cli").id, "my-app");
        assert_eq!(find_template(&views, "my-app").id, "my-app");
    }
}
