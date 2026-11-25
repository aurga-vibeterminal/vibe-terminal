export interface McpConfig {
  enabled: boolean;
  port: number;
  bindAddress: string;
  accessToken: string;
  streamThrottleMs: number;
}

/** One MCP endpoint currently served by the MCP host. */
export interface McpEndpoint {
  /** Service path segment (builtins) or addon id; served at `/{id}/mcp`. */
  id: string;
  /** Human-readable label (server name for builtins, addon name for addons). */
  name: string;
  kind: 'builtin' | 'addon';
  /** Addon endpoints only: daemon connected to the addon bridge right now. */
  daemonConnected?: boolean;
}

export interface McpTemplate {
  id: string;
  displayName: string;
  executable: string;
  /** setup template; placeholders {executable} {url} {token}; block {#token}...{/token} included only when token non-empty */
  template: string;
  /** optional setup notes for apps needing more than the template line
   * (chat-prompt setup, environment prerequisites); shown when the app is selected */
  description: string;
  /** true for server built-in templates */
  builtin: boolean;
  /** true when a built-in template has a user override */
  modified: boolean;
  /** server-rendered setup line with current values */
  preview: string;
}

export interface McpTemplateUpdate {
  displayName: string;
  executable: string;
  template: string;
}
