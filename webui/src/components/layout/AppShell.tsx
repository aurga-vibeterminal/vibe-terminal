import { Sidebar } from './Sidebar';
import { Header } from './Header';
import { MainContent } from './MainContent';
import { useConfig } from '@/context/config-context';

export function AppShell() {
  const { sidebarOpen, setSidebarOpen } = useConfig();

  return (
    <div className="flex h-screen overflow-hidden bg-warm-ivory">
      {sidebarOpen && (
        <div
          className="fixed inset-0 z-30 bg-warm-black/50 md:hidden"
          onClick={() => setSidebarOpen(false)}
        />
      )}
      <Sidebar />
      <div className="flex flex-col flex-1 min-w-0">
        <Header />
        <MainContent />
      </div>
    </div>
  );
}
