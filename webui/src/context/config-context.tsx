import { createContext, useContext, useState, useCallback, type ReactNode } from 'react';
import type { ConfigSection } from '@/types';

/** Every navigable section, used to validate the `?section=` URL param. */
const CONFIG_SECTIONS: readonly ConfigSection[] = [
  'server',
  'certificates',
  'remote-desktop',
  'terminal',
  'port-forwarding',
  'addons',
  'llm',
  'llm-stats',
  'voice',
  'ddns',
  'uart',
  'environment',
  'mcp',
  'help',
  'static-http',
];

/**
 * Initial section from the `?section=` URL param — the server opens the UI
 * at `?section=help` on first start so newcomers land on the guide. Unknown
 * or missing params fall back to the server section.
 */
function initialSection(): ConfigSection {
  if (typeof window === 'undefined') return 'server';
  const param = new URLSearchParams(window.location.search).get('section');
  if (param && (CONFIG_SECTIONS as readonly string[]).includes(param)) {
    return param as ConfigSection;
  }
  return 'server';
}

interface ConfigContextValue {
  activeSection: ConfigSection;
  setActiveSection: (section: ConfigSection) => void;
  sidebarOpen: boolean;
  setSidebarOpen: (open: boolean) => void;
  toggleSidebar: () => void;
}

const ConfigContext = createContext<ConfigContextValue | null>(null);

export function ConfigProvider({ children }: { children: ReactNode }) {
  const [activeSection, setActiveSectionState] = useState<ConfigSection>(initialSection);
  const [sidebarOpen, setSidebarOpen] = useState(false);

  const setActiveSection = useCallback((section: ConfigSection) => {
    setActiveSectionState(section);
    setSidebarOpen(false);
  }, []);

  const toggleSidebar = useCallback(() => setSidebarOpen((prev) => !prev), []);

  return (
    <ConfigContext.Provider value={{ activeSection, setActiveSection, sidebarOpen, setSidebarOpen, toggleSidebar }}>
      {children}
    </ConfigContext.Provider>
  );
}

export function useConfig() {
  const ctx = useContext(ConfigContext);
  if (!ctx) throw new Error('useConfig must be used within ConfigProvider');
  return ctx;
}
