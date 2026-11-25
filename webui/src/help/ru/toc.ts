import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'Начало работы',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'Обзор', file: 'start-overview.md' },
      { id: 'start-connect', title: 'Подключение клиента', file: 'start-connect.md' },
      { id: 'start-webui', title: 'Работа с веб-интерфейсом', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'Релей',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'Обзор', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'Руководство по настройке', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'Сертификаты и TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'Обзор', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'Домены и управление SAN', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'Строгая проверка на клиентах', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'Строгая проверка релея', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'Терминалы',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'Работа с терминалами', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'Клиент командной строки', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'Автоподтверждение', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL в Windows', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'Быстрые команды', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'Переменные окружения', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'UART-порт', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Терминал MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'Контекст для агентов', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'Скриншоты и изображения', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'История код-агента', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'Файлы',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'Обзор', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'Проводник файлов', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'Загрузка и скачивание', file: 'files-transfer.md' },
      { id: 'files-http', title: 'Публикация через HTTP-сервер', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'Обзор', file: 'git-overview.md' },
      { id: 'git-llm', title: 'Помощь LLM', file: 'git-llm.md' },
      { id: 'git-staging', title: 'Индексация и коммиты', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktree', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'Интерактивный rebase', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'Разрешение конфликтов', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push, checkout и другое', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'Удалённый рабочий стол и симуляторы',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'Обзор', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'Настройка сервера', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Устройства Android', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'Работа с клиентом', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'Десктопный клиент', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'Голосовой ввод',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'Настройка сервера', file: 'voice-server.md' },
      { id: 'voice-client', title: 'Работа с клиентом', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'Удалённый доступ',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'Проброс портов', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'ИИ-функции',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: 'Перевод',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: 'Обзор', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'Настройка сервера', file: 'translation-server.md' },
      { id: 'translation-client', title: 'Работа с клиентом', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'Аддоны',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'Обзор', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'Настройка и разработка', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'FAQ', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
