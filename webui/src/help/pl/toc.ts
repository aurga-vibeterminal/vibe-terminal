import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'Pierwsze kroki',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'Przegląd', file: 'start-overview.md' },
      { id: 'start-connect', title: 'Łączenie klienta', file: 'start-connect.md' },
      { id: 'start-webui', title: 'Korzystanie z Web UI', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'Przekaźnik (Relay)',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'Przegląd', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'Przewodnik konfiguracji', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'Certyfikaty i TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'Przegląd', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'Domeny i zarządzanie SAN', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'Ścisła weryfikacja na klientach', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'Ścisła weryfikacja na przekaźniku', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'Terminale',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'Korzystanie z terminali', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'Klient wiersza poleceń', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'Automatyczne potwierdzanie', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL w Windows', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'Szybkie polecenia', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'Zmienne środowiskowe', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'Port szeregowy UART', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Terminal MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'Dodawanie kontekstu dla agentów', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'Zrzuty ekranu i obrazy', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'Historia agenta kodu', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'Pliki',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'Przegląd', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'Eksplorator plików', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'Przesyłanie plików', file: 'files-transfer.md' },
      { id: 'files-http', title: 'Publikacja przez HTTP', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'Przegląd', file: 'git-overview.md' },
      { id: 'git-llm', title: 'Wsparcie LLM', file: 'git-llm.md' },
      { id: 'git-staging', title: 'Indeksowanie i commity', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktree', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'Interaktywny rebase', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'Rozwiązywanie konfliktów', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push, checkout i więcej', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'Pulpit zdalny i symulatory',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'Przegląd', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'Konfiguracja serwera', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Urządzenia Android', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'Korzystanie z klienta', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'Klient desktopowy', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'Wprowadzanie głosowe',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'Konfiguracja serwera', file: 'voice-server.md' },
      { id: 'voice-client', title: 'Korzystanie z klienta', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'Dostęp zdalny',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'Przekierowanie portów', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'Funkcje AI',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: 'Tłumaczenie',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: 'Przegląd', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'Konfiguracja serwera', file: 'translation-server.md' },
      { id: 'translation-client', title: 'Korzystanie z klienta', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'Dodatki',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'Przegląd', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'Dostosowywanie i rozwój', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'FAQ', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
