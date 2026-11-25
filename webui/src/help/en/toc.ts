import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'Getting Started',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'Overview', file: 'start-overview.md' },
      { id: 'start-connect', title: 'Connecting a Client', file: 'start-connect.md' },
      { id: 'start-webui', title: 'Using the Web UI', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'Relay',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'Overview', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'Setup Guide', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'Certificates & TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'Overview', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'Domains & SAN Management', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'Strict Verification on Clients', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'Strict Verification on the Relay', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'Terminals',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'Using Terminals', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'Command-line Client', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'Auto Confirm', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL on Windows', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'Quick Commands', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'Environment Variables', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'UART Serial', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Terminal MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'Adding Context to Agents', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'Screenshots & Images', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'Code Agent History', file: 'terminal-agent-history.md' },

    ],
  },
  {
    id: 'files',
    title: 'Files',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'Overview', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'File Explorer', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'Upload & Download', file: 'files-transfer.md' },
      { id: 'files-http', title: 'Host with HTTP Server', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'Overview', file: 'git-overview.md' },
      { id: 'git-llm', title: 'LLM Assistance', file: 'git-llm.md' },
      { id: 'git-staging', title: 'Staging & Committing', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktrees', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'Interactive Rebase', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'Resolving Conflicts', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push, Checkout & More', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'Remote Desktop & Simulator',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'Overview', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'Server Setup', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Android Devices', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'Using the Client', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'Desktop Client', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'Voice Input',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'Server Setup', file: 'voice-server.md' },
      { id: 'voice-client', title: 'Using the Client', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'Remote Access',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'Port Forwarding', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'AI Features',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: 'Translation',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: 'Overview', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'Server Setup', file: 'translation-server.md' },
      { id: 'translation-client', title: 'Using the Client', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'Addons',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'Overview', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'Customization & Development', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'FAQ', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
