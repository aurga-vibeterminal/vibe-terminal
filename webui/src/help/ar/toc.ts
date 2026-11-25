import type { HelpTocNode } from '../types';

export default [
  {
    id: 'start',
    title: 'البداية',
    icon: 'rocket',
    children: [
      { id: 'start-overview', title: 'نظرة عامة', file: 'start-overview.md' },
      { id: 'start-connect', title: 'توصيل عميل', file: 'start-connect.md' },
      { id: 'start-webui', title: 'استخدام واجهة الويب', file: 'start-webui.md' },
    ],
  },
  {
    id: 'relay',
    title: 'المرحّل (Relay)',
    icon: 'network',
    children: [
      { id: 'relay-overview', title: 'نظرة عامة', file: 'relay-overview.md' },
      { id: 'relay-setup', title: 'دليل الإعداد', file: 'relay-setup.md' },
    ],
  },
  {
    id: 'certificates',
    title: 'الشهادات و TLS',
    icon: 'shield',
    children: [
      { id: 'cert-overview', title: 'نظرة عامة', file: 'cert-overview.md' },
      { id: 'cert-sans', title: 'إدارة النطاقات و SAN', file: 'cert-sans.md' },
      { id: 'cert-client', title: 'التحقق الصارم على العملاء', file: 'cert-client.md' },
      { id: 'cert-relay', title: 'التحقق الصارم على المرحّل', file: 'cert-relay.md' },
    ],
  },
  {
    id: 'terminal',
    title: 'الطرفيات',
    icon: 'terminal',
    children: [
      { id: 'terminal-usage', title: 'استخدام الطرفيات', file: 'terminal-usage.md' },
      { id: 'terminal-cli', title: 'عميل سطر الأوامر', file: 'terminal-cli.md' },
      { id: 'terminal-auto-confirm', title: 'التأكيد التلقائي', file: 'terminal-auto-confirm.md' },
      { id: 'terminal-wsl', title: 'WSL على ويندوز', file: 'terminal-wsl.md' },
      { id: 'terminal-commands', title: 'الأوامر السريعة', file: 'terminal-commands.md' },
      { id: 'terminal-env', title: 'متغيرات البيئة', file: 'terminal-env.md' },
      { id: 'terminal-uart', title: 'المنفذ التسلسلي UART', file: 'terminal-uart.md' },
      { id: 'terminal-mcp', title: 'طرفية MCP', file: 'terminal-mcp.md' },
      { id: 'terminal-context', title: 'إضافة سياق للوكلاء', file: 'terminal-context.md' },
      { id: 'terminal-images', title: 'لقطات الشاشة والصور', file: 'terminal-images.md' },
      { id: 'terminal-agent-history', title: 'سجل وكيل الشيفرة', file: 'terminal-agent-history.md' },
    ],
  },
  {
    id: 'files',
    title: 'الملفات',
    icon: 'folder',
    children: [
      { id: 'files-overview', title: 'نظرة عامة', file: 'files-overview.md' },
      { id: 'files-explorer', title: 'مستكشف الملفات', file: 'files-explorer.md' },
      { id: 'files-transfer', title: 'الرفع والتنزيل', file: 'files-transfer.md' },
      { id: 'files-http', title: 'النشر عبر خادم HTTP', file: 'files-http.md' },
    ],
  },
  {
    id: 'git',
    title: 'Git',
    icon: 'git-branch',
    children: [
      { id: 'git-overview', title: 'نظرة عامة', file: 'git-overview.md' },
      { id: 'git-llm', title: 'مساعدة LLM', file: 'git-llm.md' },
      { id: 'git-staging', title: 'التحضير والتنفيذ', file: 'git-staging.md' },
      { id: 'git-worktree', title: 'أشجار العمل', file: 'git-worktree.md' },
      { id: 'git-rebase', title: 'إعادة التأسيس التفاعلية', file: 'git-rebase.md' },
      { id: 'git-conflicts', title: 'حل التعارضات', file: 'git-conflicts.md' },
      { id: 'git-push', title: 'الدفع والسحب والمزيد', file: 'git-push.md' },
    ],
  },
  {
    id: 'remote-desktop',
    title: 'سطح المكتب البعيد والمحاكيات',
    icon: 'monitor',
    children: [
      { id: 'rd-overview', title: 'نظرة عامة', file: 'remote-desktop-overview.md' },
      { id: 'rd-server', title: 'إعداد الخادم', file: 'remote-desktop-server.md' },
      { id: 'rd-android', title: 'أجهزة أندرويد', file: 'remote-desktop-android.md' },
      { id: 'rd-client', title: 'استخدام العميل', file: 'remote-desktop-client.md' },
      { id: 'rd-desktop-client', title: 'عميل سطح المكتب', file: 'remote-desktop-desktop-client.md' },
    ],
  },
  {
    id: 'voice',
    title: 'الإدخال الصوتي',
    icon: 'mic',
    children: [
      { id: 'voice-server', title: 'إعداد الخادم', file: 'voice-server.md' },
      { id: 'voice-client', title: 'استخدام العميل', file: 'voice-client.md' },
    ],
  },
  {
    id: 'remote',
    title: 'الوصول البعيد',
    icon: 'globe',
    children: [
      { id: 'remote-ddns', title: 'DDNS', file: 'remote-ddns.md' },
      { id: 'port-forwarding', title: 'إعادة توجيه المنافذ', file: 'port-forwarding.md' },
    ],
  },
  {
    id: 'ai',
    title: 'ميزات الذكاء الاصطناعي',
    icon: 'brain',
    children: [
      { id: 'ai-llm', title: 'LLM', file: 'ai-llm.md' },
      { id: 'ai-mcp', title: 'MCP', file: 'ai-mcp.md' },
    ],
  },
  {
    id: 'translation',
    title: 'الترجمة',
    icon: 'languages',
    children: [
      { id: 'translation-overview', title: 'نظرة عامة', file: 'translation-overview.md' },
      { id: 'translation-server', title: 'إعداد الخادم', file: 'translation-server.md' },
      { id: 'translation-client', title: 'استخدام العميل', file: 'translation-client.md' },
    ],
  },
  {
    id: 'addon',
    title: 'الإضافات',
    icon: 'puzzle',
    children: [
      { id: 'addon-overview', title: 'نظرة عامة', file: 'addon-overview.md' },
      { id: 'addon-dev', title: 'التخصيص والتطوير', file: 'addon-dev.md' },
    ],
  },
  { id: 'faq', title: 'الأسئلة الشائعة', icon: 'circle-help', file: 'faq.md' },
] satisfies HelpTocNode[];
