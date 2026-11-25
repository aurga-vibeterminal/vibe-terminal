/**
 * URL-driven theming for the embedded mobile WebView.
 *
 * The Android/iOS client opens this UI with three optional query params so
 * the webui matches the app's settings (see ClientSettingsScreen.kt):
 *
 *   ?theme=default|frost|aurora|nebula|anime|cosmos|cyber|inkpaint
 *   &colormode=light|dark|system   (the app also sends "Light/Dark/System")
 *   &lang=<BCP-47 tag>             (handled in src/i18n)
 *
 * Params win over localStorage for the session and are persisted so in-app
 * navigation keeps the same look. When a param is absent the previous
 * localStorage value (or the system preference) is used.
 */

export type WindowTheme =
  | 'default'
  | 'frost'
  | 'aurora'
  | 'nebula'
  | 'anime'
  | 'cosmos'
  | 'cyber'
  | 'inkpaint';

export const WINDOW_THEMES: readonly WindowTheme[] = [
  'default',
  'frost',
  'aurora',
  'nebula',
  'anime',
  'cosmos',
  'cyber',
  'inkpaint',
];

export type ColorMode = 'light' | 'dark' | 'system';

const WINDOW_THEME_STORAGE_KEY = 'vt-window-theme';
const COLOR_MODE_STORAGE_KEY = 'vt-color-mode';

export interface UrlThemeConfig {
  /** Window theme id from ?theme=, null when the param is absent/invalid. */
  windowTheme: WindowTheme | null;
  /** Color mode from ?colormode=, null when the param is absent/invalid. */
  colorMode: ColorMode | null;
}

function normalizeWindowTheme(raw: string): WindowTheme | null {
  const t = raw.trim().toLowerCase();
  // "vibe" is the former name of the default theme — kept as an alias so
  // values stored by older clients keep working.
  if (t === 'vibe') return 'default';
  return (WINDOW_THEMES as readonly string[]).includes(t) ? (t as WindowTheme) : null;
}

function normalizeColorMode(raw: string): ColorMode | null {
  const t = raw.trim().toLowerCase();
  if (t === 'light' || t === 'dark' || t === 'system') return t;
  return null;
}

/** Read the ?theme= and ?colormode= params (case-insensitive values). */
export function readUrlThemeConfig(): UrlThemeConfig {
  if (typeof window === 'undefined') return { windowTheme: null, colorMode: null };
  const params = new URLSearchParams(window.location.search);
  return {
    windowTheme: params.has('theme') ? normalizeWindowTheme(params.get('theme') ?? '') : null,
    colorMode: params.has('colormode') ? normalizeColorMode(params.get('colormode') ?? '') : null,
  };
}

/**
 * True when the page was opened by the mobile client with any of the
 * appearance/language URL params (?theme, ?colormode, ?lang). The app owns
 * those settings for the session, so the webui hides its own language and
 * theme controls (see Header).
 */
export function hasClientUrlParams(): boolean {
  if (typeof window === 'undefined') return false;
  const params = new URLSearchParams(window.location.search);
  return params.has('theme') || params.has('colormode') || params.has('lang');
}

/** Stored window theme, if valid. */
export function readStoredWindowTheme(): WindowTheme | null {
  try {
    return normalizeWindowTheme(localStorage.getItem(WINDOW_THEME_STORAGE_KEY) ?? '');
  } catch {
    return null;
  }
}

export function storeWindowTheme(theme: WindowTheme): void {
  try {
    localStorage.setItem(WINDOW_THEME_STORAGE_KEY, theme);
  } catch {
    /* private mode etc. — params still apply for this session */
  }
}

/** Stored color mode ('light' | 'dark' | 'system'), if valid. */
export function readStoredColorMode(): ColorMode | null {
  try {
    return normalizeColorMode(localStorage.getItem(COLOR_MODE_STORAGE_KEY) ?? '');
  } catch {
    return null;
  }
}

export function storeColorMode(mode: ColorMode): void {
  try {
    localStorage.setItem(COLOR_MODE_STORAGE_KEY, mode);
  } catch {
    /* ignore */
  }
}

/** Resolve the effective light/dark appearance for a color mode. */
export function resolveAppearance(mode: ColorMode): 'light' | 'dark' {
  if (mode !== 'system') return mode;
  return typeof window !== 'undefined' &&
    window.matchMedia('(prefers-color-scheme: dark)').matches
    ? 'dark'
    : 'light';
}
