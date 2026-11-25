use serde::{Deserialize, Serialize};

use crate::config::types::{ConfigError, McpTemplateConfig, TerminalServerConfig};
use crate::mcp::templates::{self, McpTemplateView};

use super::super::server::HttpResponse;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebMcpTemplate {
    id: String,
    display_name: String,
    executable: String,
    template: String,
    /// Optional setup notes for apps whose flow needs more than the template
    /// line (chat-prompt setup, environment prerequisites). Shown in the webui.
    description: String,
    builtin: bool,
    modified: bool,
    /// Server-rendered setup line for this template (single source of truth
    /// for what would be typed into the terminal).
    preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) struct WebMcpTemplateUpdate {
    display_name: String,
    executable: String,
    template: String,
    /// Notes are read-only in the webui: an update that omits this field
    /// keeps the effective notes (the existing user entry's, or the
    /// builtin's when overriding one); an explicit value replaces them.
    #[serde(default)]
    description: Option<String>,
}

fn template_to_web(view: &McpTemplateView, config: &TerminalServerConfig) -> WebMcpTemplate {
    WebMcpTemplate {
        id: view.id.clone(),
        display_name: view.config.display_name.clone(),
        executable: view.config.executable.clone(),
        template: view.config.template.clone(),
        description: view.config.description.clone(),
        builtin: view.builtin,
        modified: view.modified,
        preview: templates::render_for_view(view, &view.config.executable, &config.mcp),
    }
}

pub(super) fn list_templates(config: &TerminalServerConfig) -> Result<HttpResponse, ConfigError> {
    let web: Vec<WebMcpTemplate> = templates::merged_templates(&config.mcp)
        .iter()
        .map(|view| template_to_web(view, config))
        .collect();
    Ok(HttpResponse::json(&web)?)
}

/// Validate and upsert one template entry. Builtins are shadowed by the entry
/// rather than mutated.
pub(super) fn apply_template_update(
    config: &mut TerminalServerConfig,
    id: &str,
    payload: WebMcpTemplateUpdate,
) -> Result<(), ConfigError> {
    if id.is_empty() || id.contains('/') {
        return Err(ConfigError::BadRequest(format!(
            "invalid MCP template id: {id:?}"
        )));
    }
    let display_name = payload.display_name.trim();
    if display_name.is_empty() {
        return Err(ConfigError::BadRequest(
            "MCP template display name must not be empty".to_string(),
        ));
    }
    let executable = payload.executable.trim();
    if executable.is_empty() {
        return Err(ConfigError::BadRequest(
            "MCP template executable must not be empty".to_string(),
        ));
    }
    // No {url} requirement: direct-connect templates should reference it, but
    // plugin-install or chat-prompt templates may not.
    let description = match payload.description {
        Some(d) => d.trim().to_string(),
        None => templates::merged_templates(&config.mcp)
            .iter()
            .find(|view| view.id == id)
            .map(|view| view.config.description.clone())
            .unwrap_or_default(),
    };
    config.mcp.templates.insert(
        id.to_string(),
        McpTemplateConfig {
            display_name: display_name.to_string(),
            executable: executable.to_string(),
            template: payload.template,
            description,
        },
    );
    Ok(())
}

/// The refreshed web view for one id, after a successful upsert.
pub(super) fn template_response(
    config: &TerminalServerConfig,
    id: &str,
) -> Result<HttpResponse, ConfigError> {
    let views = templates::merged_templates(&config.mcp);
    let view = views.iter().find(|view| view.id == id).ok_or_else(|| {
        ConfigError::Validation("updated MCP template was not persisted".to_string())
    })?;
    Ok(HttpResponse::json(&template_to_web(view, config))?)
}

/// Remove the user entry for `id`: deletes user apps and resets builtin
/// overrides (the builtin shows through again).
pub(super) fn delete_template(
    config: &mut TerminalServerConfig,
    id: &str,
) -> Result<(), ConfigError> {
    let known = templates::merged_templates(&config.mcp)
        .iter()
        .any(|view| view.id == id);
    if !known {
        return Err(ConfigError::BadRequest(format!(
            "unknown MCP template id: {id:?}"
        )));
    }
    config.mcp.templates.remove(id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kimi_update(description: Option<String>) -> WebMcpTemplateUpdate {
        WebMcpTemplateUpdate {
            display_name: "Kimi Code".to_string(),
            executable: "kimi".to_string(),
            template: "/mcp-config add an HTTP MCP, url is {url}".to_string(),
            description,
        }
    }

    /// The webui shows notes read-only and never sends them back; overriding
    /// a builtin must therefore keep the builtin's notes, while an explicit
    /// payload value replaces them.
    #[test]
    fn update_omitting_description_keeps_effective_notes() {
        let mut config = TerminalServerConfig::default();

        apply_template_update(&mut config, "kimi", kimi_update(None)).unwrap();
        let override_entry = config.mcp.templates.get("kimi").unwrap();
        assert!(
            !override_entry.description.is_empty(),
            "builtin notes survive an override that omits description"
        );

        apply_template_update(
            &mut config,
            "kimi",
            kimi_update(Some("custom notes".to_string())),
        )
        .unwrap();
        assert_eq!(
            config.mcp.templates.get("kimi").unwrap().description,
            "custom notes"
        );

        // Omitting again keeps the user entry's own value, not the builtin's.
        apply_template_update(&mut config, "kimi", kimi_update(None)).unwrap();
        assert_eq!(
            config.mcp.templates.get("kimi").unwrap().description,
            "custom notes"
        );
    }

    /// A brand-new user app has no notes to inherit; omitting description
    /// must not fail or invent any.
    #[test]
    fn new_app_without_description_gets_empty_notes() {
        let mut config = TerminalServerConfig::default();
        let update = WebMcpTemplateUpdate {
            display_name: "My App".to_string(),
            executable: "myapp".to_string(),
            template: "myapp mcp add {url}".to_string(),
            description: None,
        };
        apply_template_update(&mut config, "my-app", update).unwrap();
        assert_eq!(config.mcp.templates.get("my-app").unwrap().description, "");
    }
}
