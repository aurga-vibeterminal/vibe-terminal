import { createContext, useContext, useState, useCallback, useEffect, type ReactNode } from 'react';
import {
  type WindowTheme,
  type ColorMode,
  readUrlThemeConfig,
  readStoredWindowTheme,
  storeWindowTheme,
  readStoredColorMode,
  storeColorMode,
  resolveAppearance,
} from '../theme-url';

type Theme = 'light' | 'dark';

interface ThemeContextValue {
  theme: Theme;
  /** Color mode (light/dark/system); `theme` is the resolved appearance. */
  mode: ColorMode;
  /** Active window theme id, or null for the default (Aurora) palette. */
  windowTheme: WindowTheme | null;
  toggleTheme: () => void;
  setTheme: (theme: Theme) => void;
  setMode: (mode: ColorMode) => void;
  setWindowTheme: (theme: WindowTheme | null) => void;
}

const ThemeContext = createContext<ThemeContextValue>({
  theme: 'light',
  mode: 'system',
  windowTheme: null,
  toggleTheme: () => {},
  setTheme: () => {},
  setMode: () => {},
  setWindowTheme: () => {},
});

function getInitialWindowTheme(): WindowTheme | null {
  // URL param wins (mobile client), then the last stored value.
  return readUrlThemeConfig().windowTheme ?? readStoredWindowTheme();
}

function getInitialMode(): ColorMode {
  const fromUrl = readUrlThemeConfig().colorMode;
  if (fromUrl) return fromUrl;
  const stored = readStoredColorMode();
  if (stored) return stored;
  // Legacy key from the previous light/dark-only toggle.
  try {
    const legacy = localStorage.getItem('vt-theme');
    if (legacy === 'light' || legacy === 'dark') return legacy;
  } catch {
    /* ignore */
  }
  return 'system';
}

function applyToDocument(theme: Theme, windowTheme: WindowTheme | null): void {
  const root = document.documentElement;
  root.classList.remove('light', 'dark');
  root.classList.add(theme);
  if (windowTheme) {
    root.setAttribute('data-window-theme', windowTheme);
  } else {
    root.removeAttribute('data-window-theme');
  }
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [mode, setModeState] = useState<ColorMode>(getInitialMode);
  const [windowTheme, setWindowThemeState] = useState<WindowTheme | null>(getInitialWindowTheme);
  const [theme, setThemeState] = useState<Theme>(() => resolveAppearance(getInitialMode()));

  // Initial application + persistence of the URL-provided values.
  useEffect(() => {
    const appearance = resolveAppearance(mode);
    applyToDocument(appearance, windowTheme);
    if (windowTheme) storeWindowTheme(windowTheme);
    storeColorMode(mode);
    // Keep the legacy key in sync so older entry points stay consistent.
    try {
      localStorage.setItem('vt-theme', appearance);
    } catch {
      /* ignore */
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Follow the OS preference while in system mode.
  useEffect(() => {
    if (mode !== 'system' || typeof window === 'undefined') return;
    const media = window.matchMedia('(prefers-color-scheme: dark)');
    const onChange = () => {
      const next = resolveAppearance('system');
      applyToDocument(next, windowTheme);
      setThemeState(next);
    };
    media.addEventListener('change', onChange);
    return () => media.removeEventListener('change', onChange);
  }, [mode, windowTheme]);

  const setMode = useCallback(
    (m: ColorMode) => {
      const next = resolveAppearance(m);
      applyToDocument(next, windowTheme);
      storeColorMode(m);
      try {
        localStorage.setItem('vt-theme', next);
      } catch {
        /* ignore */
      }
      setModeState(m);
      setThemeState(next);
    },
    [windowTheme],
  );

  const setWindowTheme = useCallback(
    (t: WindowTheme | null) => {
      applyToDocument(resolveAppearance(mode), t);
      if (t) storeWindowTheme(t);
      setWindowThemeState(t);
    },
    [mode],
  );

  const setTheme = useCallback(
    (t: Theme) => {
      applyToDocument(t, windowTheme);
      try {
        localStorage.setItem('vt-theme', t);
      } catch {
        /* ignore */
      }
      storeColorMode(t);
      setModeState(t);
      setThemeState(t);
    },
    [windowTheme],
  );

  const toggleTheme = useCallback(() => {
    setTheme(theme === 'light' ? 'dark' : 'light');
  }, [theme, setTheme]);

  return (
    <ThemeContext.Provider
      value={{ theme, mode, windowTheme, toggleTheme, setTheme, setMode, setWindowTheme }}
    >
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider');
  return ctx;
}
