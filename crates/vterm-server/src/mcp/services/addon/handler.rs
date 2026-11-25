//! Manual rmcp `ServerHandler` for MCP addons. Unlike the terminal
//! services the tool set is dynamic (declared by the addon's JS), so no
//! `#[tool_router]` macro is used: `list_tools`/`call_tool` forward into
//! [`crate::addon::mcp_executor`].
//!
//! QuickJS runtimes are `!Send`, so the whole JS call — runtime creation,
//! source eval, entry-point invocation — runs inside
//! `tokio::task::spawn_blocking`.

use std::sync::Arc;

use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool, ToolsCapability,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler};
use serde_json::Value;

use crate::addon::{run_call_tool, run_get_instructions, run_list_tools, AddonHost};
use crate::config::ConfigManager;

pub struct AddonMcpHandler {
    pub addon_host: Arc<AddonHost>,
    pub config_manager: Arc<ConfigManager>,
    pub addon_id: String,
}

impl AddonMcpHandler {
    /// Snapshot of the current server config for the executor (addon config
    /// values like the bridge timeout live there).
    fn config_snapshot(&self) -> crate::config::TerminalServerConfig {
        self.config_manager.subscribe().borrow().clone()
    }

    /// Server instructions: prefer the addon's own `get_instructions()` entry
    /// point (context-strategy spec D1); fall back to a generic sentence.
    /// Runs the QuickJS executor synchronously — the eval is sub-millisecond
    /// and `get_info` is a sync trait method.
    fn instructions(&self) -> String {
        let config = self.config_snapshot();
        run_get_instructions(&self.addon_host, &config, &self.addon_id)
            .ok()
            .filter(|text| !text.trim().is_empty())
            .unwrap_or_else(|| {
                format!(
                    "Tools provided by the '{}' MCP addon. The tool list is dynamic: \
					 it is declared by the addon's JavaScript source and may change \
					 between calls.",
                    self.addon_id
                )
            })
    }
}

/// Build one rmcp `Tool` from an addon descriptor. Descriptors carry `name`,
/// `description`, `inputSchema`, and optionally `annotations` (the standard
/// MCP hints — readOnlyHint/destructiveHint/…, context-strategy spec D2).
fn tool_from_descriptor(descriptor: &Value) -> Result<Tool, McpError> {
    let name = descriptor
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| McpError::internal_error("tool descriptor missing 'name'", None))?
        .to_string();
    let description = descriptor
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let schema = match descriptor.get("inputSchema").and_then(Value::as_object) {
        Some(map) => map.clone(),
        None => {
            serde_json::Map::from_iter([("type".to_string(), Value::String("object".to_string()))])
        }
    };
    let mut tool = Tool::new(name, description, std::sync::Arc::new(schema));
    if let Some(annotations) = descriptor
        .get("annotations")
        .and_then(|value| serde_json::from_value(value.clone()).ok())
    {
        tool.annotations = Some(annotations);
    }
    Ok(tool)
}

impl ServerHandler for AddonMcpHandler {
    fn get_info(&self) -> ServerInfo {
        let capabilities = ServerCapabilities::builder()
            .enable_tools_with(ToolsCapability {
                list_changed: Some(false),
            })
            .build();
        ServerInfo::new(capabilities)
            .with_server_info(Implementation::new(
                format!("vterm-addon-mcp-{}", self.addon_id),
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(self.instructions())
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        async move {
            let addon_host = Arc::clone(&self.addon_host);
            let addon_id = self.addon_id.clone();
            let config = self.config_snapshot();
            let raw = tokio::task::spawn_blocking(move || {
                run_list_tools(&addon_host, &config, &addon_id)
            })
            .await
            .map_err(|e| McpError::internal_error(format!("executor join failed: {e}"), None))?
            .map_err(|e| McpError::internal_error(format!("list_tools failed: {e}"), None))?;

            let descriptors: Vec<Value> = serde_json::from_str(&raw).map_err(|e| {
                McpError::internal_error(format!("addon returned invalid tool list: {e}"), None)
            })?;

            let mut tools = Vec::with_capacity(descriptors.len());
            for descriptor in descriptors {
                tools.push(tool_from_descriptor(&descriptor)?);
            }
            Ok(ListToolsResult {
                tools,
                ..Default::default()
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            let name = request.name.to_string();
            let args_json = match &request.arguments {
                Some(map) => serde_json::to_string(map).unwrap_or_else(|_| "{}".to_string()),
                None => "{}".to_string(),
            };

            let addon_host = Arc::clone(&self.addon_host);
            let addon_id = self.addon_id.clone();
            let config = self.config_snapshot();
            let raw = tokio::task::spawn_blocking(move || {
                run_call_tool(&addon_host, &config, &addon_id, &name, &args_json)
            })
            .await
            .map_err(|e| McpError::internal_error(format!("executor join failed: {e}"), None))?
            // Tool execution failures are reported as error results, not
            // protocol errors.
            .unwrap_or_else(|e| {
                serde_json::json!({
                    "content": [{"type": "text", "text": e}],
                    "isError": true
                })
                .to_string()
            });

            let parsed: Value = serde_json::from_str(&raw).unwrap_or_else(|_| {
				serde_json::json!({
					"content": [{"type": "text", "text": format!("addon returned invalid result JSON: {raw}")}],
					"isError": true
				})
			});

            let mut content = Vec::new();
            for item in parsed
                .get("content")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
            {
                match item.get("type").and_then(Value::as_str) {
                    Some("image") => {
                        let data = item.get("data").and_then(Value::as_str).unwrap_or_default();
                        let mime = item
                            .get("mimeType")
                            .and_then(Value::as_str)
                            .unwrap_or("image/png");
                        content.push(Content::image(data, mime));
                    }
                    _ => {
                        let text = item.get("text").and_then(Value::as_str).unwrap_or_default();
                        content.push(Content::text(text));
                    }
                }
            }
            if content.is_empty() {
                content.push(Content::text("(no content)"));
            }

            if parsed.get("isError").and_then(Value::as_bool) == Some(true) {
                Ok(CallToolResult::error(content))
            } else {
                Ok(CallToolResult::success(content))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_builds_tool_with_annotations() {
        let descriptor = serde_json::json!({
            "name": "sch_list_nets",
            "description": "list nets",
            "inputSchema": { "type": "object", "properties": {} },
            "annotations": { "readOnlyHint": true }
        });
        let tool = tool_from_descriptor(&descriptor).expect("tool");
        assert_eq!(tool.name.as_ref(), "sch_list_nets");
        let annotations = tool.annotations.expect("annotations passed through");
        assert_eq!(annotations.read_only_hint, Some(true));
    }

    #[test]
    fn descriptor_without_annotations_and_schema_gets_defaults() {
        let descriptor = serde_json::json!({ "name": "demo" });
        let tool = tool_from_descriptor(&descriptor).expect("tool");
        assert_eq!(tool.name.as_ref(), "demo");
        assert!(tool.annotations.is_none());
        assert_eq!(tool.input_schema.get("type").unwrap(), "object");
    }

    #[test]
    fn descriptor_without_name_is_rejected() {
        let descriptor = serde_json::json!({ "description": "no name" });
        assert!(tool_from_descriptor(&descriptor).is_err());
    }
}
