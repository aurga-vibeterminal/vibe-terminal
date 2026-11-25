import { ThemeProvider } from '@/context/theme-context';
import { I18nProvider } from '@/i18n';
import { ConfigProvider } from '@/context/config-context';
import { ConnectionProvider } from '@/context/connection-context';
import { ToastProvider } from '@/components/shared/Toast';
import { AppShell } from '@/components/layout/AppShell';

export default function App() {
  return (
    <ThemeProvider>
      <I18nProvider>
        <ToastProvider>
          <ConfigProvider>
            <ConnectionProvider>
              <AppShell />
            </ConnectionProvider>
          </ConfigProvider>
        </ToastProvider>
      </I18nProvider>
    </ThemeProvider>
  );
}
