//! rmcp `ServerHandler` adapter — the only module that imports rmcp's
//! service-trait glue.
//!
//! ## Why `ServerHandler` is implemented directly on `TerminalTools`
//!
//! `#[tool_handler(router = <expr>)]` generates a `call_tool` that builds a
//! `ToolCallContext::new(self, request, context)` (binding the context's `S`
//! parameter to the handler type `Self`) and then calls `#router.call(tcc)`.
//! `ToolRouter<S>::call` requires `ToolCallContext<'_, S>` — i.e. the router's
//! `S` must equal the handler type. The `#[tool_router]` macro on
//! `TerminalTools` generates `tool_router() -> ToolRouter<TerminalTools>`, so
//! the router is bound to `TerminalTools`.
//!
//! If we introduced a `TerminalMcpHandler { tools: TerminalTools }` wrapper,
//! the generated `call_tool` would create a `ToolCallContext<'_, TerminalMcpHandler>`
//! while `self.tools.tool_router()` would be a `ToolRouter<TerminalTools>` — the
//! types don't line up and compilation fails. Implementing `ServerHandler`
//! directly on `TerminalTools` (with the macro's default `router =
//! Self::tool_router()`) keeps both on the same type and compiles cleanly. We
//! still override `get_info` to advertise a custom server name/instructions;
//! the macro leaves a user-provided `get_info` untouched (verified in
//! `rmcp-macros-1.8.0/src/tool_handler.rs`: the `get_info` generator is gated
//! behind `if !has_method("get_info", ...)`).
//!
//! ## Deferred: live-screen resource subscription (Phase 1.5)
//!
//! Wiring the SSE subscription for `terminal://<id>/screen/live` (see
//! [`super::resources::stream_screen`]) into the rmcp `ServerHandler` requires
//! implementing `read_resource` / `subscribe_resource` and bridging the
//! `stream_screen` mpsc output to the rmcp notification channel. That API is
//! not exercised in the rmcp 1.8.0 examples and needs dedicated exploration, so
//! it is intentionally NOT wired here. The synchronous `get_screen` tool fully
//! satisfies the "normal" screen-read path; streaming lands in a follow-up.

use rmcp::{
    model::{Implementation, ServerCapabilities, ServerInfo, ToolsCapability},
    tool_handler,
};

use super::tools::TerminalTools;

/// `ServerHandler` for terminal sessions.
///
/// `#[tool_handler]` (with its default `router = Self::tool_router()`)
/// auto-generates `call_tool`, `list_tools`, and `get_tool` from
/// [`TerminalTools::tool_router`]. Because we define `get_info` below, the
/// macro does **not** overwrite it.
///
/// [`TerminalTools::tool_router`]: super::tools::TerminalTools
#[tool_handler]
impl rmcp::ServerHandler for TerminalTools {
    fn get_info(&self) -> ServerInfo {
        // `ServerInfo` and `ServerCapabilities` are both `#[non_exhaustive]`,
        // so we go through their builder APIs rather than struct literals.
        let capabilities = ServerCapabilities::builder()
            .enable_tools_with(ToolsCapability {
                list_changed: Some(false),
            })
            .build();
        ServerInfo::new(capabilities)
            .with_server_info(Implementation::new(
                "vterm-terminal-mcp",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Exposes terminal sessions. Use list_terminals to discover IDs, \
                 get_screen to read the visible screen, send_input/send_keys to \
                 type, get_scrollback for history, clear_screen to blank the \
                 screen and scrollback, resize_terminal to resize, \
                 wait_for_prompt to block until idle, and get_terminal_info for \
                 metadata. Live screen streaming via \
                 terminal://<id>/screen/live is not yet wired (Phase 1.5).",
            )
    }
}
