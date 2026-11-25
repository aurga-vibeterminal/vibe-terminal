//! MCP tool definitions for terminal operations.
//!
//! Each tool is an async method on [`TerminalTools`], which holds shared access
//! to the [`SessionManager`]. The `#[tool_router]` attribute (applied WITHOUT the
//! `server_handler` flag) generates a `tool_router()` accessor returning a
//! `ToolRouter<TerminalTools>`. The `ServerHandler` impl in [`handler`] plugs
//! that into a custom handler that also overrides `get_info`.
//!
//! ## Verified rmcp 1.8.0 macro syntax
//!
//! - `#[tool_router]` on an inherent impl: appends a `tool_router()` method and
//!   does NOT emit `impl ServerHandler`. To get the full handler, the sibling
//!   module writes:
//!   `#[tool_handler(router = tools.tool_router())] impl ServerHandler for ... {}`
//!   (note: `tool_handler` takes `router = <expr>`; the router expr must yield a
//!   `ToolRouter<Self>`). See `rmcp-macros-1.8.0/src/tool_handler.rs`.
//! - `#[tool(name = "...", description = "...")]` on each fn. The only valid attr
//!   keys are: `name, title, description, input_schema, output_schema, annotations,
//!   execution, icons, meta, local`. There is NO `param`/`aggr` key — the draft
//!   was stale.
//! - ALL parameters must be wrapped in `Parameters<T>` where `T: Deserialize +
//!   JsonSchema`. A bare `terminal_id: String` arg is NOT given an input schema;
//!   the macro only detects a `Parameters<...>`-typed arg (see
//!   `rmcp-macros-1.8.0/src/common.rs::find_parameters_type_in_sig`). So define a
//!   small `...Params` struct per tool.
//! - Return type: `Result<Json<Output>, ErrorData>` (or bare `Json<Output>`).
//!   `Json<T>` places the value in `structured_content` with a generated output
//!   schema.
//! - Imports: `rmcp::{handler::server::wrapper::{Json, Parameters}, model::ErrorData,
//!   schemars, tool, tool_router}`.

use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    model::ErrorData,
    schemars, tool, tool_router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vterm_common::{SessionAppStatus, SessionId, SessionInfo, TerminalSize};

use crate::session::SessionManager as VtSessionManager;

use super::screen::snapshot_from_terminal;
use crate::mcp::shared::{now_iso, parse_session_id};

// ----------------------------------------------------------------------------
// Output types
// ----------------------------------------------------------------------------

/// High-level description of one terminal session, exposed via MCP.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct TerminalInfo {
    pub id: String,
    pub title: String,
    pub cwd: Option<String>,
    pub os: Option<String>,
    pub app: String,
    pub app_status: String,
    pub sequence: u64,
}

impl TerminalInfo {
    fn from_session_info(s: &SessionInfo) -> Self {
        Self {
            id: s.id.0.to_string(),
            title: s.title.clone(),
            cwd: s.current_working_path.clone(),
            os: s.terminal_os.map(|os| format!("{os:?}").to_lowercase()),
            app: format!("{:?}", s.app).to_lowercase(),
            app_status: format!("{:?}", s.app_status).to_lowercase(),
            sequence: s.sequence,
        }
    }
}

/// Object-rooted wrapper for the `list_terminals` result.
///
/// The MCP specification requires a tool's `outputSchema` to have a root type of
/// `object`. A bare `Vec<TerminalInfo>` derives an `array` root schema, which
/// rmcp rejects at runtime. Wrapping the list in a struct yields an object-rooted
/// schema (`{"terminals": [...]}`).
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct TerminalList {
    pub terminals: Vec<TerminalInfo>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct TerminalSizeOut {
    pub rows: u16,
    pub cols: u16,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ScreenResult {
    pub text: String,
    pub cursor: Cursor,
    pub size: TerminalSizeOut,
    pub timestamp: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ScrollbackResult {
    pub lines: Vec<String>,
    pub start_line: usize,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Success {
    pub success: bool,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct WaitResult {
    pub ready: bool,
    pub last_line: String,
}

// ----------------------------------------------------------------------------
// Parameter types
// ----------------------------------------------------------------------------

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetScreenParams {
    /// Terminal ID (SessionId UUID).
    pub terminal_id: String,
    /// Include scrollback history. Default false.
    #[serde(default)]
    pub include_scrollback: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetScrollbackParams {
    /// Terminal ID (SessionId UUID).
    pub terminal_id: String,
    /// Maximum number of history lines to return (default 200, capped at 1000).
    #[serde(default)]
    pub max_lines: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendInputParams {
    /// Terminal ID (SessionId UUID).
    pub terminal_id: String,
    /// Raw text to write to the PTY, byte-for-byte. No trailing newline is
    /// added — use this for partial input, prompt replies (e.g. `y`), or
    /// filling in fields. To execute a shell command, use `send_command`.
    pub text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendCommandParams {
    /// Terminal ID (SessionId UUID).
    pub terminal_id: String,
    /// A command line to execute. A carriage return (`\r`) is appended
    /// automatically so the shell runs it.
    pub command: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendKeysParams {
    /// Terminal ID (SessionId UUID).
    pub terminal_id: String,
    /// Named key, e.g. "Enter", "Ctrl+C", "Up". See [`key_name_to_bytes`].
    pub keys: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ResizeTerminalParams {
    /// Terminal ID (SessionId UUID).
    pub terminal_id: String,
    pub rows: u16,
    pub cols: u16,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WaitForPromptParams {
    /// Terminal ID (SessionId UUID).
    pub terminal_id: String,
    /// Timeout in milliseconds (default 10000).
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

/// Thin newtype wrapper so a single-field param struct still serializes as
/// `{"terminal_id": "..."}` rather than a bare string. Used by
/// `get_terminal_info`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TerminalIdParams {
    /// Terminal ID (SessionId UUID).
    pub terminal_id: String,
}

// ----------------------------------------------------------------------------
// Handler
// ----------------------------------------------------------------------------

/// Stateful holder for shared terminal-session access, exposed as MCP tools.
///
/// Use `TerminalTools::tool_router()` (generated by `#[tool_router]`) to obtain
/// a `ToolRouter<TerminalTools>` and feed it into the `ServerHandler` impl in
/// [`handler`] (which also supplies a custom `get_info`).
#[derive(Clone)]
pub struct TerminalTools {
    pub session_manager: Arc<VtSessionManager>,
}

// `vis = "pub(crate)"` makes the generated `tool_router()` accessor reachable
// from the sibling `handler` module where the `ServerHandler` impl lives.
// `server_handler` is intentionally NOT set: we want only the router, so the
// handler module can supply its own custom `get_info`.
#[tool_router(vis = "pub(crate)")]
impl TerminalTools {
    #[tool(
        name = "list_terminals",
        description = "List all open terminal sessions."
    )]
    pub async fn list_terminals(&self) -> Result<Json<TerminalList>, ErrorData> {
        let sessions = self
            .session_manager
            .list_sessions()
            .await
            .map_err(|e| ErrorData::internal_error(format!("{e:?}"), None))?;
        let terminals = sessions
            .iter()
            .map(TerminalInfo::from_session_info)
            .collect();
        Ok(Json(TerminalList { terminals }))
    }

    #[tool(
        name = "get_terminal_info",
        description = "Get details (title, cwd, app status) of one terminal."
    )]
    pub async fn get_terminal_info(
        &self,
        Parameters(TerminalIdParams { terminal_id }): Parameters<TerminalIdParams>,
    ) -> Result<Json<TerminalInfo>, ErrorData> {
        let id = parse_session_id(&terminal_id)?;
        let sessions = self
            .session_manager
            .list_sessions()
            .await
            .map_err(|e| ErrorData::internal_error(format!("{e:?}"), None))?;
        let s = sessions
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| ErrorData::invalid_params("terminal not found", None))?;
        Ok(Json(TerminalInfo::from_session_info(s)))
    }

    #[tool(
        name = "get_screen",
        description = "Get the current visible screen text of a terminal, with cursor position and size."
    )]
    pub async fn get_screen(
        &self,
        Parameters(params): Parameters<GetScreenParams>,
    ) -> Result<Json<ScreenResult>, ErrorData> {
        let id = parse_session_id(&params.terminal_id)?;
        let session = self
            .session_manager
            .get_session(id)
            .await
            .ok_or_else(|| ErrorData::invalid_params("terminal not found", None))?;
        let snap = {
            let s = session.lock().await;
            snapshot_from_terminal(&s.terminal)
        };
        // `include_scrollback` is accepted for API symmetry; the visible screen
        // already represents the live grid. (A dedicated get_scrollback tool
        // covers history.)
        let _ = params.include_scrollback;
        Ok(Json(ScreenResult {
            text: snap.text,
            cursor: Cursor {
                row: snap.cursor_row,
                col: snap.cursor_col,
            },
            size: TerminalSizeOut {
                rows: snap.rows,
                cols: snap.cols,
            },
            timestamp: now_iso(),
        }))
    }

    #[tool(
        name = "get_scrollback",
        description = "Get recent scrollback history lines from a terminal."
    )]
    pub async fn get_scrollback(
        &self,
        Parameters(GetScrollbackParams {
            terminal_id,
            max_lines,
        }): Parameters<GetScrollbackParams>,
    ) -> Result<Json<ScrollbackResult>, ErrorData> {
        let id = parse_session_id(&terminal_id)?;
        let max = max_lines.unwrap_or(200).min(1000);
        let session = self
            .session_manager
            .get_session(id)
            .await
            .ok_or_else(|| ErrorData::invalid_params("terminal not found", None))?;
        let (start, lines) = {
            let s = session.lock().await;
            s.terminal.latest_history_page(max)
        };
        let text: Vec<String> = lines
            .iter()
            .map(|row| {
                let mut line = String::new();
                for cell in row {
                    if !cell.is_wide_continuation() {
                        line.push(cell.character);
                    }
                }
                line
            })
            .collect();
        Ok(Json(ScrollbackResult {
            lines: text,
            start_line: start,
        }))
    }

    #[tool(
        name = "clear_screen",
        description = "Clear a terminal's screen buffer: blank the visible screen and purge scrollback history (like the `clear` command). The session itself keeps running — useful to drop stale output before capturing fresh screen content with get_screen."
    )]
    pub async fn clear_screen(
        &self,
        Parameters(TerminalIdParams { terminal_id }): Parameters<TerminalIdParams>,
    ) -> Result<Json<Success>, ErrorData> {
        let id = parse_session_id(&terminal_id)?;
        self.session_manager
            .clear_terminal_buffer(id)
            .await
            .map_err(|e| ErrorData::internal_error(format!("{e:?}"), None))?;
        Ok(Json(Success { success: true }))
    }

    #[tool(
        name = "send_input",
        description = "Send raw text to a terminal's PTY exactly as given (no trailing newline). Use for partial input, prompt replies (e.g. confirming with `y`), or filling in fields. To run a shell command, prefer send_command."
    )]
    pub async fn send_input(
        &self,
        Parameters(SendInputParams { terminal_id, text }): Parameters<SendInputParams>,
    ) -> Result<Json<Success>, ErrorData> {
        let id = parse_session_id(&terminal_id)?;
        self.session_manager
            .forward_input(id, None, text.into_bytes())
            .await
            .map_err(|e| ErrorData::internal_error(format!("{e:?}"), None))?;
        Ok(Json(Success { success: true }))
    }

    #[tool(
        name = "send_command",
        description = "Execute a command line in a terminal. Appends a carriage return so the shell runs it. Use this for commands; use send_input for raw text without Enter."
    )]
    pub async fn send_command(
        &self,
        Parameters(SendCommandParams {
            terminal_id,
            command,
        }): Parameters<SendCommandParams>,
    ) -> Result<Json<Success>, ErrorData> {
        let id = parse_session_id(&terminal_id)?;
        let mut bytes = command.into_bytes();
        bytes.push(b'\r');
        self.session_manager
            .forward_input(id, None, bytes)
            .await
            .map_err(|e| ErrorData::internal_error(format!("{e:?}"), None))?;
        Ok(Json(Success { success: true }))
    }

    #[tool(
        name = "send_keys",
        description = "Send named keys (Enter, Ctrl+C, Up, etc.) to a terminal."
    )]
    pub async fn send_keys(
        &self,
        Parameters(SendKeysParams { terminal_id, keys }): Parameters<SendKeysParams>,
    ) -> Result<Json<Success>, ErrorData> {
        let bytes = key_name_to_bytes(&keys)
            .ok_or_else(|| ErrorData::invalid_params(format!("unknown key: {keys}"), None))?;
        let id = parse_session_id(&terminal_id)?;
        self.session_manager
            .forward_input(id, None, bytes)
            .await
            .map_err(|e| ErrorData::internal_error(format!("{e:?}"), None))?;
        Ok(Json(Success { success: true }))
    }

    #[tool(name = "resize_terminal", description = "Resize a terminal's PTY.")]
    pub async fn resize_terminal(
        &self,
        Parameters(ResizeTerminalParams {
            terminal_id,
            rows,
            cols,
        }): Parameters<ResizeTerminalParams>,
    ) -> Result<Json<Success>, ErrorData> {
        let id = parse_session_id(&terminal_id)?;
        // resize_session requires a viewer tx to authorize the resize; create a
        // throwaway channel that is never read.
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        self.session_manager
            .resize_session(id, &tx, TerminalSize::new(rows, cols))
            .await
            .map_err(|e| ErrorData::internal_error(format!("{e:?}"), None))?;
        Ok(Json(Success { success: true }))
    }

    #[tool(
        name = "wait_for_prompt",
        description = "Wait until the terminal is no longer busy (command finished). Polls app_status."
    )]
    pub async fn wait_for_prompt(
        &self,
        Parameters(WaitForPromptParams {
            terminal_id,
            timeout_ms,
        }): Parameters<WaitForPromptParams>,
    ) -> Result<Json<WaitResult>, ErrorData> {
        let id = parse_session_id(&terminal_id)?;
        let timeout = std::time::Duration::from_millis(timeout_ms.unwrap_or(10_000));
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let session = self
                .session_manager
                .get_session(id)
                .await
                .ok_or_else(|| ErrorData::invalid_params("terminal not found", None))?;
            let s = session.lock().await;
            let busy = s.app_status == SessionAppStatus::Busy;
            if !busy {
                let last_line = s.terminal.current_line_text();
                return Ok(Json(WaitResult {
                    ready: true,
                    last_line,
                }));
            }
            drop(s);
            if std::time::Instant::now() >= deadline {
                let last_line = session.lock().await.terminal.current_line_text();
                return Ok(Json(WaitResult {
                    ready: false,
                    last_line,
                }));
            }
            drop(session);
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
}

/// Translate a named key into the bytes a terminal expects.
///
/// Recognized names: `Enter`/`Return`, `Tab`, `Esc`/`Escape`, `Backspace`,
/// arrow keys (`Up`/`Down`/`Left`/`Right`), `Home`, `End`, `PgUp`, `PgDn`, and
/// `Ctrl+<letter>` (e.g. `Ctrl+C` → `0x03`).
pub(crate) fn key_name_to_bytes(name: &str) -> Option<Vec<u8>> {
    let bytes: Vec<u8> = match name {
        "Enter" | "Return" => vec![b'\r'],
        "Tab" => vec![b'\t'],
        "Esc" | "Escape" => vec![0x1b],
        "Backspace" => vec![0x7f],
        "Up" => vec![0x1b, b'[', b'A'],
        "Down" => vec![0x1b, b'[', b'B'],
        "Right" => vec![0x1b, b'[', b'C'],
        "Left" => vec![0x1b, b'[', b'D'],
        "Home" => vec![0x1b, b'[', b'H'],
        "End" => vec![0x1b, b'[', b'F'],
        "PgUp" => vec![0x1b, b'[', b'5', b'~'],
        "PgDn" => vec![0x1b, b'[', b'6', b'~'],
        _ => {
            let (ctrl, rest) = name.split_once('+')?;
            if ctrl.eq_ignore_ascii_case("ctrl") && rest.len() == 1 {
                let c = rest.chars().next()?.to_ascii_lowercase();
                if c.is_ascii_lowercase() {
                    vec![(c as u8) - b'a' + 1]
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
    };
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_keys() {
        assert_eq!(key_name_to_bytes("Enter"), Some(vec![b'\r']));
        assert_eq!(key_name_to_bytes("Return"), Some(vec![b'\r']));
        assert_eq!(key_name_to_bytes("Ctrl+C"), Some(vec![3]));
        assert_eq!(key_name_to_bytes("Ctrl+c"), Some(vec![3]));
        assert_eq!(key_name_to_bytes("Up"), Some(vec![0x1b, b'[', b'A']));
        assert_eq!(key_name_to_bytes("Tab"), Some(vec![b'\t']));
        assert_eq!(key_name_to_bytes("Esc"), Some(vec![0x1b]));
        assert_eq!(key_name_to_bytes("garbage"), None);
        assert_eq!(key_name_to_bytes("Ctrl+1"), None);
        assert_eq!(key_name_to_bytes("Shift+A"), None);
    }

    #[test]
    fn parse_session_id_rejects_garbage() {
        // invalid_params error is constructed but here we just check it's an Err.
        assert!(parse_session_id("not-a-uuid").is_err());
        assert!(parse_session_id("00000000-0000-0000-0000-000000000000").is_ok());
    }
}
