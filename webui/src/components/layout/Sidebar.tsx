import {
  Server,
  Monitor,
  Terminal,
  ArrowLeftRight,
  Puzzle,
  Brain,
  BarChart3,
  Mic,
  Globe,
  Usb,
  Trees,
  Plug,
  CircleHelp,
  FolderUp,
  ShieldCheck,
  X,
} from 'lucide-react';
import { cn } from '@/lib/cn';
import { useConfig } from '@/context/config-context';
import { useTranslation } from '@/i18n';
import type { ConfigSection } from '@/types';

const navItems: { id: ConfigSection; labelKey?: string; label?: string; icon: typeof Server }[] = [
  { id: 'server', labelKey: 'nav.server', icon: Server },
  { id: 'certificates', labelKey: 'nav.certificates', icon: ShieldCheck },
  { id: 'terminal', label: 'Terminal', icon: Terminal },
  { id: 'uart', label: 'UART', icon: Usb },
  { id: 'addons', labelKey: 'nav.addons', icon: Puzzle },
  { id: 'llm', label: 'LLM', icon: Brain },
  { id: 'voice', labelKey: 'nav.voice', icon: Mic },
  { id: 'environment', labelKey: 'nav.environment', icon: Trees },
  { id: 'mcp', label: 'MCP', icon: Plug },
  { id: 'ddns', label: 'DDNS', icon: Globe },
  { id: 'port-forwarding', labelKey: 'nav.portForwarding', icon: ArrowLeftRight },
  { id: 'static-http', labelKey: 'nav.staticHttp', icon: FolderUp },
  { id: 'remote-desktop', labelKey: 'nav.remoteDesktop', icon: Monitor },
  { id: 'llm-stats', labelKey: 'nav.llmStats', icon: BarChart3 },
];

export function Sidebar() {
  const { activeSection, setActiveSection, sidebarOpen, setSidebarOpen } = useConfig();
  const { t } = useTranslation();

  return (
    <aside
      className={cn(
        'w-60 h-screen bg-warm-ivory flex flex-col shrink-0',
        'fixed md:static z-40 transition-transform duration-200',
        'border-r border-golden-sand',
        sidebarOpen ? 'translate-x-0' : '-translate-x-full md:translate-x-0',
      )}
    >
      <div className="p-4 flex items-center justify-between">
        <div>
          <h1 className="text-sm font-semibold text-warm-black">
            AURGA Vibe Terminal
          </h1>
          <p className="text-xs text-warm-khaki mt-0.5">
            {t('app.subtitle')}
          </p>
        </div>
        <div className="flex items-center gap-1">
          {/* Icon-only help entry — sits at the top right of the header.
              On narrow screens the sidebar is hidden and the same icon
              lives in the Header's top-right controls instead. */}
          <button
            type="button"
            onClick={() => setActiveSection('help')}
            title={t('nav.help')}
            aria-label={t('nav.help')}
            className={cn(
              'p-1.5 rounded-standard transition-colors duration-vt-base',
              activeSection === 'help'
                ? 'bg-accent-subtle text-accent-text'
                : 'text-deep-olive hover:text-warm-black hover:bg-pale-sand',
            )}
          >
            <CircleHelp className="w-4 h-4" />
          </button>
          <button
            type="button"
            onClick={() => setSidebarOpen(false)}
            className="md:hidden p-1 rounded-tight text-warm-khaki hover:text-warm-black transition-colors duration-vt-base"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>
      <nav className="flex-1 px-3 py-2 space-y-0.5">
        {navItems.map(({ id, labelKey, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setActiveSection(id)}
            className={cn(
              'w-full flex items-center gap-3 px-3 py-2 rounded-standard text-xs font-medium transition-colors duration-vt-base',
              activeSection === id
                ? 'bg-accent-subtle text-accent-text'
                : 'text-deep-olive hover:text-warm-black hover:bg-pale-sand',
            )}
          >
            <Icon className="w-3.5 h-3.5 shrink-0" />
            {label ?? t(labelKey ?? '')}
          </button>
        ))}
      </nav>
      <div className="p-4">
        <p className="text-[11px] text-warm-khaki">
          {`v${__APP_VERSION__}`}
        </p>
      </div>
    </aside>
  );
}
