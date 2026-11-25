import { Menu, Languages, Sun, Moon, CircleHelp } from 'lucide-react';
import { cn } from '@/lib/cn';
import { useConfig } from '@/context/config-context';
import { useTheme } from '@/context/theme-context';
import { SelectField } from '@/components/shared/SelectField';
import { localeNames, useTranslation } from '@/i18n';
import { hasClientUrlParams } from '@/theme-url';

const sectionTitleKeys: Record<string, string> = {
  server: 'server.title',
  'remote-desktop': 'remoteDesktop.title',
  'port-forwarding': 'portForward.title',
  ddns: 'ddns.title',
  environment: 'environment.title',
  help: 'help.title',
  'static-http': 'staticHttp.title',
};

export function Header() {
  const { activeSection, setActiveSection, toggleSidebar } = useConfig();
  const { theme, toggleTheme } = useTheme();
  const { t, locale, setLocale, locales } = useTranslation();
  // When the mobile client opened this page with ?lang/?theme/?colormode,
  // the app owns those settings for the session — hide our own controls so
  // the two sides can't fight over the same appearance.
  const clientControlled = hasClientUrlParams();

  return (
    <header className="h-10 bg-warm-ivory flex items-center justify-between px-4 shrink-0 border-b border-golden-sand">
      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={toggleSidebar}
          className="md:hidden p-1.5 rounded-standard text-warm-black hover:bg-pale-sand transition-colors duration-vt-base"
        >
          <Menu className="w-4 h-4" />
        </button>
        <h2 className="text-sm font-medium text-warm-black">
          {t(sectionTitleKeys[activeSection])}
        </h2>
      </div>
      <div className="flex items-center gap-2">
        {/* Help entry for narrow screens, where the sidebar (and its help
            icon) is hidden — placed before the language control. */}
        <button
          type="button"
          onClick={() => setActiveSection('help')}
          title={t('nav.help')}
          aria-label={t('nav.help')}
          className={cn(
            'md:hidden p-1.5 rounded-standard transition-colors duration-vt-base',
            activeSection === 'help'
              ? 'bg-accent-subtle text-accent-text'
              : 'text-deep-olive hover:text-warm-black hover:bg-pale-sand',
          )}
        >
          <CircleHelp className="w-3.5 h-3.5" />
        </button>
        {!clientControlled && (
          <div className="flex items-center gap-2">
            <div className="flex items-center gap-1.5 p-1.5 rounded-standard text-deep-olive hover:bg-pale-sand transition-colors duration-vt-base text-xs font-medium">
              <Languages className="w-3.5 h-3.5" />
              <span className="sr-only">Language</span>
              <SelectField
                ghost
                value={locale}
                onChange={(event) => setLocale(event.target.value as typeof locale)}
                options={locales.map((option) => ({ value: option, label: localeNames[option] }))}
                ariaLabel="Language"
              />
            </div>
            <button
              type="button"
              onClick={toggleTheme}
              className="p-1.5 rounded-standard text-deep-olive hover:bg-pale-sand transition-colors duration-vt-base"
              title={theme === 'light' ? t('header.switchToDark') : t('header.switchToLight')}
            >
              {theme === 'light' ? <Moon className="w-3.5 h-3.5" /> : <Sun className="w-3.5 h-3.5" />}
            </button>
          </div>
        )}
      </div>
    </header>
  );
}
