import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'Bắt đầu',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'Tổng quan', file: 'start-overview.md' },
      { id: 'start-connect', title: 'Kết nối máy khách', file: 'start-connect.md' },
      { id: 'start-webui', title: 'Sử dụng Web UI', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'Relay',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'Tổng quan', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'Hướng dẫn thiết lập', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'Chứng chỉ & TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'Tổng quan', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'Quản lý tên miền & SAN', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'Xác minh nghiêm ngặt trên máy khách', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'Xác minh nghiêm ngặt trên relay', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'Thiết bị đầu cuối',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'Sử dụng terminal', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'Máy khách dòng lệnh', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'Tự động xác nhận', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL trên Windows', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'Lệnh nhanh', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'Biến môi trường', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'Cổng nối tiếp UART', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Terminal MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'Thêm ngữ cảnh cho tác tử', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'Ảnh chụp màn hình và hình ảnh', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'Lịch sử tác tử mã', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'Tệp',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'Tổng quan', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'Trình khám phá tệp', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'Tải lên & tải xuống', file: 'files-transfer.md' },
      { id: 'files-http', title: 'Xuất bản bằng máy chủ HTTP', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'Tổng quan', file: 'git-overview.md' },
      { id: 'git-llm', title: 'Trợ giúp LLM', file: 'git-llm.md' },
      { id: 'git-staging', title: 'Staging & commit', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktree', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'Rebase tương tác', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'Giải quyết xung đột', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push, checkout và khác', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'Màn hình từ xa & trình mô phỏng',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'Tổng quan', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'Thiết lập máy chủ', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Thiết bị Android', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'Sử dụng máy khách', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'Máy khách desktop', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'Nhập bằng giọng nói',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'Thiết lập máy chủ', file: 'voice-server.md' },
      { id: 'voice-client', title: 'Sử dụng máy khách', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'Truy cập từ xa',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'Chuyển tiếp cổng', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'Tính năng AI',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: 'Dịch thuật',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: 'Tổng quan', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'Thiết lập máy chủ', file: 'translation-server.md' },
      { id: 'translation-client', title: 'Sử dụng máy khách', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'Addon',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'Tổng quan', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'Tùy chỉnh và phát triển', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'Câu hỏi thường gặp', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
