import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'Memulai',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'Ikhtisar', file: 'start-overview.md' },
      { id: 'start-connect', title: 'Menghubungkan Klien', file: 'start-connect.md' },
      { id: 'start-webui', title: 'Menggunakan Web UI', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'Relay',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'Ikhtisar', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'Panduan Penyiapan', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'Sertifikat & TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'Gambaran Umum', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'Pengelolaan Domain & SAN', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'Verifikasi Ketat di Klien', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'Verifikasi Ketat pada Relay', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'Terminal',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'Menggunakan Terminal', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'Klien Baris Perintah', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'Konfirmasi Otomatis', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL di Windows', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'Perintah Cepat', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'Variabel Lingkungan', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'Serial UART', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Terminal MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'Menambahkan Konteks ke Agen', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'Tangkapan Layar & Gambar', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'Riwayat Agen Kode', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'Berkas',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'Ikhtisar', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'Penjelajah Berkas', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'Unggah & Unduh', file: 'files-transfer.md' },
      { id: 'files-http', title: 'Publikasikan dengan Server HTTP', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'Ikhtisar', file: 'git-overview.md' },
      { id: 'git-llm', title: 'Bantuan LLM', file: 'git-llm.md' },
      { id: 'git-staging', title: 'Staging & Commit', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktree', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'Rebase Interaktif', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'Menyelesaikan Konflik', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push, Checkout & Lainnya', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'Desktop Jarak Jauh & Simulator',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'Ikhtisar', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'Penyiapan Server', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Perangkat Android', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'Menggunakan Klien', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'Klien Desktop', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'Input Suara',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'Penyiapan Server', file: 'voice-server.md' },
      { id: 'voice-client', title: 'Menggunakan Klien', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'Akses Jarak Jauh',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'Port Forwarding', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'Fitur AI',
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
      { id: 'translation-overview', title: 'Ikhtisar', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'Penyiapan Server', file: 'translation-server.md' },
      { id: 'translation-client', title: 'Menggunakan Klien', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'Addon',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'Sekilas', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'Kustomisasi dan Pengembangan', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'FAQ', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
