import { createContext, useContext, useState, useCallback, useEffect, type ReactNode } from 'react';
import en from './locales/en';
import zhCN from './locales/zh-CN';
import zhTW from './locales/zh-TW';
import de from './locales/de';
import es from './locales/es';
import fr from './locales/fr';
import ja from './locales/ja';
import ko from './locales/ko';
import nl from './locales/nl';
import pl from './locales/pl';
import pt from './locales/pt';
import ru from './locales/ru';
import th from './locales/th';
import vi from './locales/vi';
import ar from './locales/ar';
import id from './locales/id';
import ms from './locales/ms';
import uk from './locales/uk';

/**
 * Locales with a full translation file. Extend by adding the file to
 * ./locales and registering it here — `normalizeLangTag` below then picks
 * it up from the ?lang= URL param automatically.
 */
export type Locale = 'en' | 'zh-CN' | 'zh-TW' | 'de' | 'es' | 'fr' | 'ja' | 'ko' | 'nl' | 'pl' | 'pt' | 'ru' | 'th' | 'vi' | 'ar' | 'id' | 'ms' | 'uk';

const localeMap: Record<Locale, Record<string, string>> = {
  en, 'zh-CN': zhCN, 'zh-TW': zhTW, de, es, fr, ja, ko, nl, pl, pt, ru, th, vi, ar, id, ms, uk,
};

export const localeNames: Record<Locale, string> = {
  en: 'EN',
  'zh-CN': '中文',
  'zh-TW': '繁中',
  de: 'Deutsch',
  es: 'Español',
  fr: 'Français',
  ja: '日本語',
  ko: '한국어',
  nl: 'Nederlands',
  pl: 'Polski',
  pt: 'Português',
  ru: 'Русский',
  th: 'ไทย',
  vi: 'Tiếng Việt',
  ar: 'العربية',
  id: 'Bahasa Indonesia',
  ms: 'Bahasa Melayu',
  uk: 'Українська',
};

/**
 * Map an arbitrary BCP-47 language tag (the ?lang= URL param sent by the
 * mobile client, e.g. "zh", "zh-TW", "en", "ja", "de") to a locale we have
 * translations for. Unknown languages fall back to English; Traditional
 * / Hong Kong Chinese falls back to the Simplified dictionary for now.
 */
export function normalizeLangTag(tag: string): Locale {
  const t = tag.trim().toLowerCase();
  if (localeMap[t as Locale]) return t as Locale;
  // Try the primary subtag ("pt-BR" → "pt") in case a base file exists.
  const primary = t.split('-')[0];
  if (primary === 'zh') {
    return t.includes('tw') || t.includes('hk') || t.includes('mo') ? 'zh-TW' : 'zh-CN';
  }
  if (localeMap[primary as Locale]) return primary as Locale;
  return 'en';
}

interface I18nContextValue {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: string, params?: Record<string, string | number>) => string;
  locales: Locale[];
}

const I18nContext = createContext<I18nContextValue | null>(null);

function getInitialLocale(): Locale {
  if (typeof window === 'undefined') return 'en';
  // ?lang= wins (mobile client), e.g. ?lang=zh-TW — any BCP-47 tag.
  const params = new URLSearchParams(window.location.search);
  const langParam = params.get('lang');
  if (langParam) return normalizeLangTag(langParam);
  const stored = localStorage.getItem('db-locale') as Locale | null;
  if (stored && localeMap[stored]) return stored;
  return normalizeLangTag(navigator.language);
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(getInitialLocale);

  useEffect(() => {
    localStorage.setItem('db-locale', locale);
    // Keep <html lang> in sync for accessibility / font selection.
    document.documentElement.setAttribute('lang', locale);
  }, [locale]);

  const setLocale = useCallback((l: Locale) => setLocaleState(l), []);

  const t = useCallback(
    (key: string, params?: Record<string, string | number>): string => {
      let value = localeMap[locale][key] ?? localeMap.en[key] ?? key;
      if (params) {
        for (const [k, v] of Object.entries(params)) {
          value = value.replace(`{${k}}`, String(v));
        }
      }
      return value;
    },
    [locale],
  );

  return (
    <I18nContext.Provider value={{ locale, setLocale, t, locales: Object.keys(localeMap) as Locale[] }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useTranslation() {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error('useTranslation must be used within I18nProvider');
  return ctx;
}
