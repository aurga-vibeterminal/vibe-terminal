import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: '快速上手',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: '功能概览', file: 'start-overview.md' },
      { id: 'start-connect', title: '客户端配对', file: 'start-connect.md' },
      { id: 'start-webui', title: '使用 Web 界面', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: '中继',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: '功能概览', file: 'relay-overview.md' },
      { id: 'relay-setup', title: '配置指南', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: '证书与 TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: '概览', file: 'cert-overview.md' },
      { id: 'cert-sans', title: '域名与 SAN 管理', file: 'cert-sans.md' },
      { id: 'cert-client', title: '客户端严格验证', file: 'cert-client.md' },
      { id: 'cert-relay', title: '中继严格验证', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: '终端',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: '终端使用', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: '命令行客户端', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: '自动确认', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'Windows 下的 WSL', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: '快速命令', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: '环境变量', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'UART 串口', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Terminal MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: '为 Code Agent 添加上下文', file: 'terminal-context.md' },
      { id: 'terminal-images', title: '截图与图片', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'Code Agent 会话历史', file: 'terminal-agent-history.md' },

    ],
  },
  {
    id: 'files',
    title: '文件管理',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: '功能概览', file: 'files-overview.md' },
      { id: 'files-explorer', title: '文件浏览器', file: 'files-explorer.md' },
      { id: 'files-transfer', title: '上传与下载', file: 'files-transfer.md' },
      { id: 'files-http', title: 'HTTP 文件服务', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: '概览', file: 'git-overview.md' },
      { id: 'git-llm', title: 'LLM 辅助', file: 'git-llm.md' },
      { id: 'git-staging', title: '暂存与提交', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktree 工作树', file: 'git-worktree.md' },
      { id: 'git-rebase', title: '交互式 Rebase', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: '解决冲突', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push、Checkout 与更多', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: '远程桌面与模拟器',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: '功能概览', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: '服务器配置', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Android 设备', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: '客户端使用', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: '桌面客户端', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: '语音输入',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: '服务器配置', file: 'voice-server.md' },
      { id: 'voice-client', title: '客户端使用', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: '远程访问',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS 动态域名', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: '端口转发', file: 'port-forwarding.md' },
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
    title: '翻译',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: '功能概览', file: 'translation-overview.md' },
      { id: 'translation-server', title: '服务器配置', file: 'translation-server.md' },
      { id: 'translation-client', title: '客户端使用', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: '插件',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: '概述', file: 'addon-overview.md' },
      { id: 'addon-dev', title: '自定义与开发', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: '常见问题', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
