import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'Primeros pasos',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'Descripción general', file: 'start-overview.md' },
      { id: 'start-connect', title: 'Conectar un cliente', file: 'start-connect.md' },
      { id: 'start-webui', title: 'Uso de la interfaz web', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'Relay',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'Descripción general', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'Guía de configuración', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'Certificados y TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'Descripción general', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'Dominios y gestión de SAN', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'Verificación estricta en clientes', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'Verificación estricta en el relay', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'Terminales',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'Uso de terminales', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'Cliente de línea de comandos', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'Confirmación automática', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL en Windows', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'Comandos rápidos', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'Variables de entorno', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'Puerto serie UART', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'Terminal MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'Añadir contexto a los agentes', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'Capturas e imágenes', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'Historial del agente de código', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'Archivos',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'Descripción general', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'Explorador de archivos', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'Subir y descargar', file: 'files-transfer.md' },
      { id: 'files-http', title: 'Publicar con servidor HTTP', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'Descripción general', file: 'git-overview.md' },
      { id: 'git-llm', title: 'Asistencia LLM', file: 'git-llm.md' },
      { id: 'git-staging', title: 'Staging y commits', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'Worktrees', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'Rebase interactivo', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'Resolver conflictos', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'Push, checkout y más', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'Escritorio remoto y simuladores',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'Descripción general', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'Configuración del servidor', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'Dispositivos Android', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'Uso del cliente', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'Cliente de escritorio', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'Entrada de voz',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'Configuración del servidor', file: 'voice-server.md' },
      { id: 'voice-client', title: 'Uso del cliente', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'Acceso remoto',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'Redirección de puertos', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'Funciones de IA',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: 'Traducción',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: 'Descripción general', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'Configuración del servidor', file: 'translation-server.md' },
      { id: 'translation-client', title: 'Uso del cliente', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'Complementos',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'Descripción general', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'Personalización y desarrollo', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'FAQ', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
