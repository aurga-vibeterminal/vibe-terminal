use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub r#type: String,
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    pub is_error: bool,
}

impl ToolResult {
    pub fn success(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
            is_error: false,
        }
    }

    pub fn error(tool_call_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: message.into(),
            is_error: true,
        }
    }
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, args: serde_json::Value) -> Result<String, String>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.definition().function.name.clone();
        self.tools.insert(name, tool);
    }

    pub fn get_definitions(&self) -> Vec<serde_json::Value> {
        self.tools
            .values()
            .map(|t| serde_json::to_value(t.definition()).expect("tool definition serialization"))
            .collect()
    }

    pub async fn execute(&self, name: &str, args: serde_json::Value) -> Result<String, String> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| format!("Unknown tool: {name}"))?;
        tool.execute(args).await
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

pub mod builtin;

#[cfg(test)]
mod tests {
    use super::*;

    struct EchoTool;

    #[async_trait::async_trait]
    impl Tool for EchoTool {
        fn definition(&self) -> ToolDefinition {
            ToolDefinition {
                r#type: "function".into(),
                function: FunctionDefinition {
                    name: "echo".into(),
                    description: "Echo back the input".into(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "text": { "type": "string" }
                        },
                        "required": ["text"]
                    }),
                },
            }
        }

        async fn execute(&self, args: serde_json::Value) -> Result<String, String> {
            let text = args["text"].as_str().unwrap_or("");
            Ok(text.to_string())
        }
    }

    #[tokio::test]
    async fn test_registry_register_and_execute() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));

        assert!(!registry.is_empty());
        assert_eq!(registry.get_definitions().len(), 1);

        let result = registry
            .execute("echo", serde_json::json!({"text": "hello"}))
            .await
            .unwrap();
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn test_registry_unknown_tool() {
        let registry = ToolRegistry::new();
        let result = registry.execute("nonexistent", serde_json::json!({})).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_file_read_tool() {
        let dir = std::env::temp_dir().join("vterm_agent_test_file_read");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.txt");
        std::fs::write(&file_path, "line1\nline2\nline3\n").unwrap();

        let tool = builtin::FileReadTool;
        let result = tool
            .execute(serde_json::json!({
                "path": file_path.display().to_string()
            }))
            .await
            .unwrap();
        assert!(result.contains("line1"));
        assert!(result.contains("line3"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn test_file_read_with_offset_limit() {
        let dir = std::env::temp_dir().join("vterm_agent_test_offset");
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.txt");
        std::fs::write(&file_path, "a\nb\nc\nd\ne\n").unwrap();

        let tool = builtin::FileReadTool;
        let result = tool
            .execute(serde_json::json!({
                "path": file_path.display().to_string(),
                "offset": 2,
                "limit": 2
            }))
            .await
            .unwrap();
        assert!(result.contains("b"));
        assert!(result.contains("c"));
        assert!(!result.contains("a"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn test_file_find_tool() {
        let dir = std::env::temp_dir().join("vterm_agent_test_find");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("hello.rs"), "fn main() {}").unwrap();
        std::fs::write(dir.join("hello.txt"), "text").unwrap();

        let tool = builtin::FileFindTool;
        let result = tool
            .execute(serde_json::json!({
                "pattern": "*.rs",
                "directory": dir.display().to_string()
            }))
            .await
            .unwrap();
        assert!(result.contains("hello.rs"));
        assert!(!result.contains("hello.txt"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn test_register_all_builtins() {
        let mut registry = ToolRegistry::new();
        builtin::register_all(&mut registry);
        assert!(!registry.is_empty());
        assert_eq!(registry.get_definitions().len(), 4);
    }
}
