import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'Erste Schritte',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'Überblick', file: 'start-overview.md' },
      { id: 'start-connect', title: 'Client verbinden', file: 'start-connect.md' },
      { id: 'start-webui', title: 'Die Web UI verwenden', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'Relay',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'Überblick', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'Einrichtungsanleitung', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'Zertifikate & TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'Übersicht', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'Domains & SAN verwalten', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'Strikte Verifizierung auf Clients', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'Strikte Verifizierung auf dem Relay', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'Terminals',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'Terminals verwenden', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'Kommandozeilen-Client', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'Auto Confirm', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL unter Windows', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'Quick Commands', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'Umgebungsvariablen', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'UART Seriell', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Terminal MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'Kontext an Agents', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'Screenshots & Bilder', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'Code-Agent-Verlauf', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'Dateien',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'Überblick', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'Datei-Explorer', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'Hoch- & Herunterladen', file: 'files-transfer.md' },
      { id: 'files-http', title: 'Mit HTTP-Server freigeben', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'Überblick', file: 'git-overview.md' },
      { id: 'git-llm', title: 'LLM-Unterstützung', file: 'git-llm.md' },
      { id: 'git-staging', title: 'Stagen & Committen', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktrees', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'Interaktiver Rebase', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'Konflikte lösen', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push, Checkout & Mehr', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'Remote Desktop & Simulator',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'Überblick', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'Server-Setup', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Android-Geräte', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'Client verwenden', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'Desktop-Client', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'Spracheingabe',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'Server-Setup', file: 'voice-server.md' },
      { id: 'voice-client', title: 'Client verwenden', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'Fernzugriff',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'Portweiterleitung', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'KI-Funktionen',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: 'Übersetzung',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: 'Überblick', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'Server-Setup', file: 'translation-server.md' },
      { id: 'translation-client', title: 'Client verwenden', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'Add-ons',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'Überblick', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'Anpassung & Entwicklung', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'FAQ', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
