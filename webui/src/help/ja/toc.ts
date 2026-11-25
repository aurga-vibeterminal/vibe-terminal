import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'はじめに',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: '概要', file: 'start-overview.md' },
      { id: 'start-connect', title: 'クライアントの接続', file: 'start-connect.md' },
      { id: 'start-webui', title: 'Web UI の使い方', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'リレー',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: '概要', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'セットアップガイド', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: '証明書と TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: '概要', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'ドメインと SAN の管理', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'クライアントでの厳格な検証', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'リレーでの厳格な検証', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'ターミナル',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'ターミナルの使い方', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'コマンドラインクライアント', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: '自動確認', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'Windows の WSL', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'クイックコマンド', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: '環境変数', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'UART シリアル', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'ターミナル MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'エージェントへのコンテキスト追加', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'スクリーンショットと画像', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'コードエージェント履歴', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'ファイル',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: '概要', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'ファイルエクスプローラー', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'アップロードとダウンロード', file: 'files-transfer.md' },
      { id: 'files-http', title: 'HTTP サーバーで公開', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: '概要', file: 'git-overview.md' },
      { id: 'git-llm', title: 'LLM アシスト', file: 'git-llm.md' },
      { id: 'git-staging', title: 'ステージングとコミット', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'ワークツリー', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'インタラクティブリベース', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: '競合の解決', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'プッシュ・チェックアウト他', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'リモートデスクトップとシミュレーター',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: '概要', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'サーバー設定', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Android デバイス', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'クライアントの使い方', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'デスクトップクライアント', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: '音声入力',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'サーバー設定', file: 'voice-server.md' },
      { id: 'voice-client', title: 'クライアントの使い方', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'リモートアクセス',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'ポートフォワーディング', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'AI 機能',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: '翻訳',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: '概要', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'サーバー設定', file: 'translation-server.md' },
      { id: 'translation-client', title: 'クライアントの使い方', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'アドオン',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: '概要', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'カスタマイズと開発', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'FAQ', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
