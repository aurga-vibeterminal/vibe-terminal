import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: '快速上手',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: '功能總覽', file: 'start-overview.md' },
      { id: 'start-connect', title: '用戶端配對', file: 'start-connect.md' },
      { id: 'start-webui', title: '使用 Web 介面', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: '中繼',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: '功能概覽', file: 'relay-overview.md' },
      { id: 'relay-setup', title: '設定指南', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: '憑證與 TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: '概覽', file: 'cert-overview.md' },
      { id: 'cert-sans', title: '網域與 SAN 管理', file: 'cert-sans.md' },
      { id: 'cert-client', title: '用戶端嚴格驗證', file: 'cert-client.md' },
      { id: 'cert-relay', title: '中繼嚴格驗證', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: '終端機',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: '終端機使用', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: '命令列用戶端', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: '自動確認', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'Windows 下的 WSL', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: '快速命令', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: '環境變數', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'UART 串列埠', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Terminal MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: '為 Code Agent 新增上下文', file: 'terminal-context.md' },
      { id: 'terminal-images', title: '截圖與圖片', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'Code Agent 會話歷史', file: 'terminal-agent-history.md' },

    ],
  },
  {
    id: 'files',
    title: '檔案管理',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: '功能總覽', file: 'files-overview.md' },
      { id: 'files-explorer', title: '檔案瀏覽器', file: 'files-explorer.md' },
      { id: 'files-transfer', title: '上傳與下載', file: 'files-transfer.md' },
      { id: 'files-http', title: 'HTTP 檔案服務', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: '總覽', file: 'git-overview.md' },
      { id: 'git-llm', title: 'LLM 輔助', file: 'git-llm.md' },
      { id: 'git-staging', title: '暫存與提交', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktree 工作樹', file: 'git-worktree.md' },
      { id: 'git-rebase', title: '互動式 Rebase', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: '解決衝突', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push、Checkout 與更多', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: '遠端桌面與模擬器',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: '功能概覽', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: '伺服器設定', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Android 裝置', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: '用戶端使用', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: '桌面用戶端', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: '語音輸入',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: '伺服器設定', file: 'voice-server.md' },
      { id: 'voice-client', title: '用戶端使用', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: '遠端存取',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS 動態網域', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: '連接埠轉發', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'AI 功能',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM 大模型', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: '翻譯',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: '功能概覽', file: 'translation-overview.md' },
      { id: 'translation-server', title: '伺服器設定', file: 'translation-server.md' },
      { id: 'translation-client', title: '用戶端使用', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: '附加元件',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: '概覽', file: 'addon-overview.md' },
      { id: 'addon-dev', title: '自訂與開發', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: '常見問題', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
