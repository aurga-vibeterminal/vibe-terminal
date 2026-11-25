import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'Premiers pas',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'Présentation', file: 'start-overview.md' },
      { id: 'start-connect', title: 'Connecter un client', file: 'start-connect.md' },
      { id: 'start-webui', title: 'Utiliser l’interface Web', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'Relais',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'Présentation', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'Guide d’installation', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'Certificats et TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'Vue d’ensemble', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'Domaines et gestion des SAN', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'Vérification stricte côté clients', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'Vérification stricte côté relais', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'Terminaux',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'Utiliser les terminaux', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'Client en ligne de commande', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'Confirmation automatique', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL sous Windows', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'Commandes rapides', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'Variables d’environnement', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'Série UART', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Terminal MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'Ajouter du contexte aux agents', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'Captures d’écran et images', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'Historique des agents de code', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'Fichiers',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'Présentation', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'Explorateur de fichiers', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'Téléverser et télécharger', file: 'files-transfer.md' },
      { id: 'files-http', title: 'Publier avec un serveur HTTP', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'Présentation', file: 'git-overview.md' },
      { id: 'git-llm', title: 'Assistance LLM', file: 'git-llm.md' },
      { id: 'git-staging', title: 'Staging et commits', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktrees', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'Rebase interactif', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'Résoudre les conflits', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push, checkout et plus', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'Bureau à distance et simulateurs',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'Présentation', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'Configuration du serveur', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Appareils Android', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'Utiliser le client', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'Client de bureau', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'Saisie vocale',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'Configuration du serveur', file: 'voice-server.md' },
      { id: 'voice-client', title: 'Utiliser le client', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'Accès distant',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'Redirection de ports', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'Fonctions IA',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: 'Traduction',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: 'Présentation', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'Configuration du serveur', file: 'translation-server.md' },
      { id: 'translation-client', title: 'Utiliser le client', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'Addons',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'Aperçu', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'Personnalisation et développement', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'FAQ', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
