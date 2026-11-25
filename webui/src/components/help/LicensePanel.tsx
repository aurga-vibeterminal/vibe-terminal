import { useEffect, useMemo, useState } from 'react';
import {
  AlertTriangle,
  ChevronRight,
  Download,
  ExternalLink,
  Loader2,
  Package,
  Search,
} from 'lucide-react';
import { useTranslation } from '@/i18n';
import { cn } from '@/lib/cn';

/**
 * Open Source Licenses page for the Help center.
 *
 * Renders the manifest generated at build time by
 * webui/scripts/generate-licenses.mjs (npm run generate:licenses), which is
 * copied into webui/dist and embedded in the server binary — so this page
 * works offline in production and needs no backend API. Full license texts
 * are content-addressed files under /licenses/, fetched lazily when a row
 * is expanded.
 */
export interface LicenseEntry {
  origin: 'rust' | 'npm' | 'manual';
  name: string;
  version: string;
  license: string;
  /** Present when the applied license is a chosen alternative of a dual license. */
  licenseOriginal?: string;
  licenseNote?: string;
  repository: string | null;
  textFile: string | null;
}

interface LicenseManifest {
  version: number;
  generatedAt: string;
  counts: Record<string, number>;
  warnings?: string[];
  entries: LicenseEntry[];
}

const ORIGIN_ORDER: LicenseEntry['origin'][] = ['rust', 'npm', 'manual'];
const ORIGIN_LABEL_KEY: Record<LicenseEntry['origin'], string> = {
  rust: 'licenses.rust',
  npm: 'licenses.npm',
  manual: 'licenses.manual',
};

const entryKey = (e: LicenseEntry) => `${e.origin}:${e.name}:${e.version}`;

function LicenseRow({
  entry,
  isOpen,
  text,
  loading,
  onToggle,
}: {
  entry: LicenseEntry;
  isOpen: boolean;
  text: string | null;
  loading: boolean;
  onToggle: () => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="border-b border-golden-sand/50 last:border-b-0">
      <button
        type="button"
        onClick={onToggle}
        aria-expanded={isOpen}
        className="w-full flex items-center gap-2 py-1.5 px-1 text-left rounded-tight hover:bg-pale-sand/60 transition-colors duration-vt-base"
      >
        <ChevronRight
          className={cn(
            'w-3.5 h-3.5 shrink-0 text-warm-khaki transition-transform duration-vt-base',
            isOpen && 'rotate-90',
          )}
        />
        <span className="text-xs font-medium text-warm-black truncate">{entry.name}</span>
        <span className="text-2xs text-warm-khaki shrink-0">{entry.version}</span>
        <span
          title={entry.licenseNote}
          className={cn(
            'ml-auto shrink-0 text-2xs font-medium px-1.5 py-0.5 rounded-tight border border-golden-sand bg-pale-sand',
            entry.license === 'UNKNOWN' ? 'text-accent-text' : 'text-deep-olive',
          )}
        >
          {entry.license}
        </span>
        {entry.repository && (
          <a
            href={entry.repository}
            target="_blank"
            rel="noreferrer"
            onClick={(e) => e.stopPropagation()}
            className="shrink-0 p-0.5 text-warm-khaki hover:text-warm-black transition-colors duration-vt-base"
            aria-label={entry.repository}
          >
            <ExternalLink className="w-3.5 h-3.5" />
          </a>
        )}
      </button>
      {isOpen && (
        <div className="pb-2 pl-6">
          {loading ? (
            <div className="flex items-center gap-2 text-xs text-warm-khaki">
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              {t('licenses.loading')}
            </div>
          ) : text ? (
            <pre className="vt-scrollbar max-h-72 overflow-auto rounded-standard border border-golden-sand bg-pale-sand p-3 font-mono text-xs whitespace-pre-wrap break-words text-deep-olive">
              {text}
            </pre>
          ) : (
            <p className="text-xs text-warm-khaki">{t('licenses.missingText')}</p>
          )}
        </div>
      )}
    </div>
  );
}

export function LicensePanel() {
  const { t } = useTranslation();
  const [manifest, setManifest] = useState<LicenseManifest | null>(null);
  const [failed, setFailed] = useState(false);
  const [query, setQuery] = useState('');
  const [open, setOpen] = useState<string | null>(null);
  // Lazy text cache: entry textFile -> content (null while loading).
  const [texts, setTexts] = useState<Record<string, string>>({});

  useEffect(() => {
    let cancelled = false;
    fetch('/licenses.json')
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.json() as Promise<LicenseManifest>;
      })
      .then((data) => {
        if (!cancelled) setManifest(data);
      })
      .catch(() => {
        if (!cancelled) setFailed(true);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const expand = (entry: LicenseEntry) => {
    const key = entryKey(entry);
    if (open === key) {
      setOpen(null);
      return;
    }
    setOpen(key);
    if (entry.textFile && texts[entry.textFile] === undefined) {
      fetch(`/${entry.textFile}`)
        .then((res) => {
          if (!res.ok) throw new Error(`HTTP ${res.status}`);
          return res.text();
        })
        .then((text) => setTexts((prev) => ({ ...prev, [entry.textFile!]: text })))
        .catch(() => setTexts((prev) => ({ ...prev, [entry.textFile!]: '' })));
    }
  };

  const filtered = useMemo(() => {
    if (!manifest) return null;
    const q = query.trim().toLowerCase();
    if (!q) return manifest.entries;
    return manifest.entries.filter(
      (e) =>
        e.name.toLowerCase().includes(q) ||
        e.license.toLowerCase().includes(q) ||
        e.version.toLowerCase().includes(q),
    );
  }, [manifest, query]);

  if (failed) {
    return (
      <div className="flex items-start gap-2 text-sm text-deep-olive">
        <AlertTriangle className="w-4 h-4 shrink-0 mt-0.5 text-accent-text" />
        <p>{t('licenses.error')}</p>
      </div>
    );
  }

  if (!manifest || !filtered) {
    return (
      <div className="flex items-center gap-2 text-sm text-warm-khaki">
        <Loader2 className="w-4 h-4 animate-spin" />
        {t('licenses.loading')}
      </div>
    );
  }

  const generatedAt = new Date(manifest.generatedAt).toLocaleString();

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap items-center gap-2">
        <div className="relative flex-1 min-w-48 max-w-sm">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-warm-khaki pointer-events-none" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder={t('licenses.search')}
            className="w-full pl-8 pr-3 py-1.5 rounded-standard border border-golden-sand bg-warm-ivory text-xs text-warm-black placeholder:text-warm-khaki focus:outline-none focus:ring-1 focus:ring-accent-text/40"
          />
        </div>
        <a
          href="/third-party-notices.txt"
          download
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-standard text-xs font-medium text-deep-olive hover:text-warm-black hover:bg-pale-sand border border-golden-sand transition-colors duration-vt-base"
        >
          <Download className="w-3.5 h-3.5 shrink-0" />
          {t('licenses.download')}
        </a>
        <span className="text-2xs text-warm-khaki ml-auto">{t('licenses.generatedAt', { date: generatedAt })}</span>
      </div>

      {manifest.warnings && manifest.warnings.length > 0 && (
        <div className="flex items-start gap-2 rounded-standard border border-golden-sand bg-pale-sand/60 p-2.5 text-xs text-deep-olive">
          <AlertTriangle className="w-3.5 h-3.5 shrink-0 mt-0.5 text-accent-text" />
          <ul className="space-y-0.5">
            {manifest.warnings.map((w) => (
              <li key={w}>{w}</li>
            ))}
          </ul>
        </div>
      )}

      {filtered.length === 0 ? (
        <p className="text-sm text-warm-khaki">{t('licenses.noMatch')}</p>
      ) : (
        ORIGIN_ORDER.map((origin) => {
          const list = filtered.filter((e) => e.origin === origin);
          if (list.length === 0) return null;
          return (
            <section key={origin}>
              <h4 className="flex items-center gap-1.5 text-sm font-semibold text-warm-black mt-4 mb-2">
                <Package className="w-4 h-4 text-warm-khaki" />
                {t(ORIGIN_LABEL_KEY[origin])}
                <span className="text-xs font-normal text-warm-khaki">({list.length})</span>
              </h4>
              <div className="rounded-standard border border-golden-sand bg-warm-ivory px-2 py-1">
                {list.map((entry) => {
                  const key = entryKey(entry);
                  return (
                    <LicenseRow
                      key={key}
                      entry={entry}
                      isOpen={open === key}
                      text={entry.textFile ? texts[entry.textFile] ?? null : null}
                      loading={!!entry.textFile && texts[entry.textFile] === undefined}
                      onToggle={() => expand(entry)}
                    />
                  );
                })}
              </div>
            </section>
          );
        })
      )}
    </div>
  );
}
