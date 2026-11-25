import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'Primeiros passos',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'Visão geral', file: 'start-overview.md' },
      { id: 'start-connect', title: 'Conectar um cliente', file: 'start-connect.md' },
      { id: 'start-webui', title: 'Usar a interface Web', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'Relay',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'Visão geral', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'Guia de configuração', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'Certificados e TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'Visão geral', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'Domínios e gerenciamento de SAN', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'Verificação estrita em clientes', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'Verificação estrita no relay', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'Terminais',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'Usar terminais', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'Cliente de linha de comando', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'Confirmação automática', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL no Windows', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'Comandos rápidos', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'Variáveis de ambiente', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'Serial UART', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Terminal MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'Adicionar contexto aos agentes', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'Capturas de tela e imagens', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'Histórico do agente de código', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'Arquivos',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'Visão geral', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'Explorador de arquivos', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'Enviar e baixar', file: 'files-transfer.md' },
      { id: 'files-http', title: 'Publicar com servidor HTTP', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'Visão geral', file: 'git-overview.md' },
      { id: 'git-llm', title: 'Assistência LLM', file: 'git-llm.md' },
      { id: 'git-staging', title: 'Staging e commits', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktrees', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'Rebase interativo', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'Resolver conflitos', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push, checkout e mais', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'Área de trabalho remota e simuladores',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'Visão geral', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'Configuração do servidor', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Dispositivos Android', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'Usar o cliente', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'Cliente de desktop', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'Entrada de voz',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'Configuração do servidor', file: 'voice-server.md' },
      { id: 'voice-client', title: 'Usar o cliente', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'Acesso remoto',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'Encaminhamento de portas', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'Recursos de IA',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: 'Tradução',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: 'Visão geral', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'Configuração do servidor', file: 'translation-server.md' },
      { id: 'translation-client', title: 'Usar o cliente', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'Addons',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'Visão geral', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'Personalização e desenvolvimento', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'FAQ', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
