import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'Aan de slag',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'Overzicht', file: 'start-overview.md' },
      { id: 'start-connect', title: 'Een client verbinden', file: 'start-connect.md' },
      { id: 'start-webui', title: 'De Web UI gebruiken', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'Relay',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'Overzicht', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'Installatiehandleiding', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'Certificaten & TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'Overzicht', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'Domeinen & SAN-beheer', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'Stricte verificatie op clients', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'Stricte verificatie op de relay', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'Terminals',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'Terminals gebruiken', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'Commandoregel-client', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'Automatisch bevestigen', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL onder Windows', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'Snelle commando’s', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'Omgevingsvariabelen', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'UART-serieel', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Terminal MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'Context toevoegen aan agents', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'Screenshots & afbeeldingen', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'Geschiedenis code-agent', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'Bestanden',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'Overzicht', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'Bestandsverkenner', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'Uploaden & downloaden', file: 'files-transfer.md' },
      { id: 'files-http', title: 'Publiceren met HTTP-server', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'Overzicht', file: 'git-overview.md' },
      { id: 'git-llm', title: 'LLM-assistentie', file: 'git-llm.md' },
      { id: 'git-staging', title: 'Stagen & committen', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktrees', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'Interactieve rebase', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'Conflicten oplossen', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push, checkout & meer', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'Bureaublad op afstand & simulators',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'Overzicht', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'Serverinstellingen', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Android-apparaten', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'De client gebruiken', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'Desktopclient', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'Spraakinvoer',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'Serverinstellingen', file: 'voice-server.md' },
      { id: 'voice-client', title: 'De client gebruiken', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'Toegang op afstand',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'Portforwarding', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'AI-functies',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: 'Vertaling',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: 'Overzicht', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'Serverinstellingen', file: 'translation-server.md' },
      { id: 'translation-client', title: 'De client gebruiken', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'Add-ons',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'Overzicht', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'Aanpassen en ontwikkelen', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'FAQ', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
