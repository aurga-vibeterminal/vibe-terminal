// Client-side mirror of the server's MCP setup-template renderer
// (`crates/vterm-server/src/mcp/templates.rs::render_template`). Keeping a
// local copy lets the webui preview shell commands and chat prompts live from
// the current (possibly unsaved) form values instead of the last-fetched config.
// Keep both implementations in sync.

const TOKEN_BLOCK_OPEN = '{#token}';
const TOKEN_BLOCK_CLOSE = '{/token}';

/** Substitute `{executable}`, `{url}`, `{token}` and include
 * `{#token}...{/token}` blocks only when `token` is non-empty. */
export function renderMcpTemplate(
  template: string,
  executable: string,
  url: string,
  token: string,
): string {
  let out = '';
  let rest = template;
  for (;;) {
    const start = rest.indexOf(TOKEN_BLOCK_OPEN);
    if (start === -1) break;
    out += rest.slice(0, start);
    const afterOpen = rest.slice(start + TOKEN_BLOCK_OPEN.length);
    const end = afterOpen.indexOf(TOKEN_BLOCK_CLOSE);
    if (end === -1) {
      // Unclosed block: emit the remainder verbatim.
      out += rest.slice(start);
      rest = '';
      break;
    }
    if (token !== '') out += afterOpen.slice(0, end);
    rest = afterOpen.slice(end + TOKEN_BLOCK_CLOSE.length);
  }
  out += rest;
  // Replacer functions keep `$`-sequences in the values literal (a plain
  // string replacement would interpret `$&` etc.).
  return out
    .replace(/\{executable\}/g, () => executable)
    .replace(/\{url\}/g, () => url)
    .replace(/\{token\}/g, () => token)
    .trim();
}
