import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'Permulaan',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'Gambaran Keseluruhan', file: 'start-overview.md' },
      { id: 'start-connect', title: 'Menyambungkan Klien', file: 'start-connect.md' },
      { id: 'start-webui', title: 'Menggunakan Web UI', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'Relay',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'Gambaran Keseluruhan', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'Panduan Persediaan', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'Sijil & TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'Gambaran Keseluruhan', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'Pengurusan Domain & SAN', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'Pengesahan Ketat pada Klien', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'Pengesahan Ketat pada Relay', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'Terminal',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'Menggunakan Terminal', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'Klien Baris Perintah', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'Pengesahan Automatik', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL di Windows', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'Perintah Pantas', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'Pemboleh Ubah Persekitaran', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'Siri UART', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Terminal MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'Menambah Konteks kepada Ejen', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'Tangkapan Skrin & Imej', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'Sejarah Ejen Kod', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'Fail',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'Gambaran Keseluruhan', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'Penjelajah Fail', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'Muat Naik & Muat Turun', file: 'files-transfer.md' },
      { id: 'files-http', title: 'Terbitkan dengan Pelayan HTTP', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'Gambaran Keseluruhan', file: 'git-overview.md' },
      { id: 'git-llm', title: 'Bantuan LLM', file: 'git-llm.md' },
      { id: 'git-staging', title: 'Staging & Commit', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktree', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'Rebase Interaktif', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'Menyelesaikan Konflik', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push, Checkout & Lagi', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'Desktop Jauh & Simulator',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'Gambaran Keseluruhan', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'Persediaan Pelayan', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Peranti Android', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'Menggunakan Klien', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'Klien Desktop', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'Input Suara',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'Persediaan Pelayan', file: 'voice-server.md' },
      { id: 'voice-client', title: 'Menggunakan Klien', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'Akses Jauh',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'Pemajuan Port', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'Ciri AI',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: 'Terjemahan',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: 'Gambaran Keseluruhan', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'Persediaan Pelayan', file: 'translation-server.md' },
      { id: 'translation-client', title: 'Menggunakan Klien', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'Tambahan',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'Gambaran', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'Penyesuaian dan Pembangunan', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'FAQ', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
