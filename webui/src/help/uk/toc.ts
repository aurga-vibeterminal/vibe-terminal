import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'Початок роботи',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'Огляд', file: 'start-overview.md' },
      { id: 'start-connect', title: 'Підключення клієнта', file: 'start-connect.md' },
      { id: 'start-webui', title: 'Користування веб-інтерфейсом', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'Реле',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'Огляд', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'Керівництво з налаштування', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'Сертифікати та TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'Огляд', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'Домени та керування SAN', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'Сувора перевірка на клієнтах', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'Сувора перевірка релея', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'Термінали',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'Користування терміналами', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'Клієнт командного рядка', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'Автопідтвердження', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL у Windows', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'Швидкі команди', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'Змінні середовища', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'Послідовний порт UART', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Термінал MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'Додавання контексту агентам', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'Скріншоти та зображення', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'Історія код-агента', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'Файли',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'Огляд', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'Провідник файлів', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'Завантаження та отримання', file: 'files-transfer.md' },
      { id: 'files-http', title: 'Публікація через HTTP-сервер', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'Огляд', file: 'git-overview.md' },
      { id: 'git-llm', title: 'Підтримка LLM', file: 'git-llm.md' },
      { id: 'git-staging', title: 'Індексація та коміти', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktree', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'Інтерактивний rebase', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'Розв’язання конфліктів', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push, checkout та інше', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'Віддалена стільниця та симулятори',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'Огляд', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'Налаштування сервера', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Пристрої Android', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'Користування клієнтом', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'Настільний клієнт', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'Голосове введення',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'Налаштування сервера', file: 'voice-server.md' },
      { id: 'voice-client', title: 'Користування клієнтом', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'Віддалений доступ',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'Проброс портів', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'Функції ШІ',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: 'Переклад',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: 'Огляд', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'Налаштування сервера', file: 'translation-server.md' },
      { id: 'translation-client', title: 'Користування клієнтом', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'Додатки',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'Огляд', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'Налаштування та розробка', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'Часті питання', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
