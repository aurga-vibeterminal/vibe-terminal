import { useEffect, useMemo, useRef, useState } from 'react';
import {
  Brain,
  ChevronRight,
  ChevronLeft,
  CircleHelp,
  FileText,
  Folder,
  GitBranch,
  Globe,
  Image,
  Languages,
  ListTree,
  Mic,
  Monitor,
  Network,
  Puzzle,
  Rocket,
  Shield,
  Terminal,
  X,
  type LucideIcon,
} from 'lucide-react';
import ReactMarkdown, { type Components } from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { getHelpDiagram } from '@/components/help/diagrams';
import { LicensePanel } from '@/components/help/LicensePanel';
import { SectionHeader } from '@/components/shared/SectionHeader';
import {
  collectFolderIds,
  findHelpPath,
  flattenHelpLeaves,
  getHelpImage,
  getHelpToc,
  isHelpFolder,
  resolveHelpDoc,
  type HelpTocNode,
} from '@/help';
import { useTranslation } from '@/i18n';
import { cn } from '@/lib/cn';

/** Folder icon names used in the per-locale `toc.ts` files. */
const folderIcons: Record<string, LucideIcon> = {
  rocket: Rocket,
  globe: Globe,
  languages: Languages,
  terminal: Terminal,
  brain: Brain,
  network: Network,
  monitor: Monitor,
  mic: Mic,
  'git-branch': GitBranch,
  folder: Folder,
  puzzle: Puzzle,
  shield: Shield,
  'circle-help': CircleHelp,
};

interface HelpTreeRowProps {
  node: HelpTocNode;
  depth: number;
  ancestors: string[];
  selectedId: string;
  expanded: Set<string>;
  onSelect: (id: string, ancestors: string[]) => void;
  onToggle: (id: string) => void;
}

function HelpTreeRow({ node, depth, ancestors, selectedId, expanded, onSelect, onToggle }: HelpTreeRowProps) {
  const isFolder = isHelpFolder(node);
  const isOpen = expanded.has(node.id);
  const isSelected = selectedId === node.id;
  // Folders always show an icon: the named one from the toc, or a generic
  // folder glyph when the name has no mapping (a new toc icon name before
  // this map learns it must never render an icon-less row).
  const Icon = isFolder ? folderIcons[node.icon ?? ''] ?? Folder : undefined;

  return (
    <div>
      <button
        type="button"
        onClick={() => (isFolder ? onToggle(node.id) : onSelect(node.id, ancestors))}
        style={{ paddingLeft: `${depth * 12 + 8}px` }}
        className={cn(
          'w-full flex items-center gap-1.5 pr-2 py-1.5 rounded-standard text-xs font-medium transition-colors duration-vt-base text-left',
          isSelected
            ? 'bg-accent-subtle text-accent-text'
            : 'text-deep-olive hover:text-warm-black hover:bg-pale-sand',
        )}
      >
        {isFolder ? (
          <>
            <ChevronRight
              className={cn(
                'w-3 h-3 shrink-0 text-warm-khaki transition-transform duration-vt-base',
                isOpen && 'rotate-90',
              )}
            />
            {Icon && <Icon className="w-3.5 h-3.5 shrink-0" />}
          </>
        ) : (
          <FileText className="w-3.5 h-3.5 shrink-0 ml-[15px] text-warm-khaki" />
        )}
        <span className="truncate">{node.title}</span>
      </button>
      {isFolder && isOpen && (
        <div>
          {node.children.map((child) => (
            <HelpTreeRow
              key={child.id}
              node={child}
              depth={depth + 1}
              ancestors={[...ancestors, node.id]}
              selectedId={selectedId}
              expanded={expanded}
              onSelect={onSelect}
              onToggle={onToggle}
            />
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * Placeholder reserving the space of a help image that has not been
 * committed yet (`images/<file>` referenced from the markdown but shipped
 * for neither the document's locale nor the fallback locale). Span-based:
 * markdown images render inside a paragraph, so block elements (`figure`)
 * would trigger React's validateDOMNesting warning.
 */
function HelpImagePlaceholder({ caption }: { caption: string }) {
  return (
    <span className="block my-4">
      <span className="flex min-h-[120px] flex-col items-center justify-center gap-2 rounded-standard border border-dashed border-golden-sand bg-pale-sand/50">
        <Image className="h-5 w-5 text-warm-khaki" />
        <span className="px-4 text-center text-xs text-warm-khaki">{caption}</span>
      </span>
    </span>
  );
}

/** A resolved help image with its caption (the markdown alt text). */
function HelpImageFigure({ src, caption }: { src: string; caption: string }) {
  return (
    <span className="block my-4">
      <img
        src={src}
        alt={caption}
        className="w-full max-w-full rounded-standard border border-golden-sand"
      />
      <span className="mt-1.5 block text-xs text-warm-khaki">{caption}</span>
    </span>
  );
}

/**
 * A resolved help video (`.webm` / `.mp4` referenced from the markdown like
 * an image). Autoplaying, looping, silent — a drop-in motion replacement
 * for GIF heroes at a fraction of the size.
 */
function HelpVideoFigure({ src, caption }: { src: string; caption: string }) {
  return (
    <span className="block my-4">
      <video
        src={src}
        autoPlay
        loop
        muted
        playsInline
        className="w-full max-w-full rounded-standard border border-golden-sand"
      />
      <span className="mt-1.5 block text-xs text-warm-khaki">{caption}</span>
    </span>
  );
}

/** `true` for help-media files the renderer plays as video instead of showing as an image. */
function isHelpVideoFile(file: string): boolean {
  return /\.(webm|mp4)$/i.test(file);
}

/**
 * Markdown renderer for help bodies. Document files live in
 * `src/help/<locale>/<doc>.md`; relative `images/<file>` tags resolve
 * against the displayed document's locale (see [`resolveHelpDoc`]) and
 * everything else maps onto the section's Tailwind styling.
 */
function createMarkdownComponents(docLocale: string): Components {
  return {
    h1: ({ node: _, ...props }) => (
      <h1 className="text-base font-semibold text-warm-black mt-5 mb-2 first:mt-0" {...props} />
    ),
    h2: ({ node: _, ...props }) => (
      <h2 className="text-sm font-semibold text-warm-black mt-5 mb-2 first:mt-0" {...props} />
    ),
    h3: ({ node: _, ...props }) => (
      <h3 className="text-[13px] font-semibold text-warm-black mt-4 mb-1.5 first:mt-0" {...props} />
    ),
    p: ({ node: _, ...props }) => <p className="my-2 first:mt-0" {...props} />,
    ul: ({ node: _, ...props }) => <ul className="list-disc pl-5 space-y-1 my-3" {...props} />,
    ol: ({ node: _, ...props }) => <ol className="list-decimal pl-5 space-y-1 my-3" {...props} />,
    a: ({ node: _, ...props }) => (
      <a
        className="text-accent-text underline underline-offset-2 break-words"
        target="_blank"
        rel="noreferrer"
        {...props}
      />
    ),
    blockquote: ({ node: _, ...props }) => (
      <blockquote className="border-l-2 border-golden-sand pl-3 my-3 text-warm-khaki" {...props} />
    ),
    img: ({ src, alt }) => {
      const caption = alt ?? '';
      // `diagram/<name>` references render the live SVG component registered
      // under that name (see components/help/diagrams.tsx). Written path-like
      // rather than `diagram:` because react-markdown's default urlTransform
      // strips URLs with non-allowlisted URL schemes.
      if (src?.startsWith('diagram/')) {
        const Diagram = getHelpDiagram(src.slice('diagram/'.length));
        if (Diagram) {
          return (
            <span className="block my-4">
              <Diagram />
              {caption && <span className="mt-1.5 block text-xs text-warm-khaki">{caption}</span>}
            </span>
          );
        }
      }
      // Relative references point into src/help/<locale>/images/ — the
      // `images/` prefix is stripped before lookup (getHelpImage adds it
      // back). Video files (webm/mp4) render as looping muted players.
      // Absolute URLs and site-root paths render as-is.
      const file = src?.replace(/^\.\//, '').replace(/^images\//, '');
      if (file && !/^[a-z][a-z0-9+.-]*:/i.test(file) && !file.startsWith('/')) {
        const url = getHelpImage(docLocale, file);
        return url ? (
          isHelpVideoFile(file) ? (
            <HelpVideoFigure src={url} caption={caption} />
          ) : (
            <HelpImageFigure src={url} caption={caption} />
          )
        ) : (
          <HelpImagePlaceholder caption={caption} />
        );
      }
      return (
        <img
          src={src}
          alt={caption}
          className="my-4 w-full max-w-full rounded-standard border border-golden-sand"
        />
      );
    },
    code: ({ node: _, className, ...props }) =>
      className ? (
        <code className={className} {...props} />
      ) : (
        // Convention: inline `backticks` in help markdown mark UI elements
        // the reader clicks/taps (buttons, menu items, form fields) — styled
        // as a chip resembling a UI control, not code.
        <code
          className="rounded-tight border border-golden-sand bg-pale-sand px-1 py-0.5 font-semibold text-warm-black"
          {...props}
        />
      ),
    pre: ({ node: _, ...props }) => (
      <pre className="my-3 overflow-x-auto rounded-standard bg-pale-sand p-3 font-mono text-xs vt-scrollbar" {...props} />
    ),
    table: ({ node: _, ...props }) => (
      <div className="my-3 overflow-x-auto">
        <table className="w-full text-left text-xs" {...props} />
      </div>
    ),
    th: ({ node: _, ...props }) => (
      <th className="border-b border-golden-sand py-1.5 pr-3 font-semibold text-warm-black" {...props} />
    ),
    td: ({ node: _, ...props }) => <td className="border-b border-golden-sand/50 py-1.5 pr-3" {...props} />,
    hr: ({ node: _, ...props }) => <hr className="my-4 border-golden-sand" {...props} />,
  };
}

function HelpMarkdown({ markdown, docLocale }: { markdown: string; docLocale: string }) {
  const components = useMemo(() => createMarkdownComponents(docLocale), [docLocale]);
  return (
    <div className="text-sm leading-vt-body text-deep-olive">
      <ReactMarkdown remarkPlugins={[remarkGfm]} components={components}>
        {markdown}
      </ReactMarkdown>
    </div>
  );
}

/** Synthetic leaf id for the Open Source Licenses page (see HelpSection). */
const LICENSES_DOC_ID = 'oss-licenses';

export function HelpSection() {
  const { t, locale } = useTranslation();
  // The Open Source Licenses page is data-driven (the JSON manifest that
  // scripts/generate-licenses.mjs emits into webui/public at build time),
  // not a per-locale markdown document — so instead of repeating it in
  // every locale's toc.ts it is appended here with its i18n title and
  // rendered by LicensePanel below.
  const toc = useMemo(
    () => [...getHelpToc(locale), { id: LICENSES_DOC_ID, title: t('licenses.title'), file: '' }],
    [locale, t],
  );
  const leaves = useMemo(() => flattenHelpLeaves(toc), [toc]);
  const [selectedId, setSelectedId] = useState('');
  // Unknown ids (e.g. right after switching to a locale with a different
  // tree) fall back to the first document.
  const effectiveId = leaves.some((leaf) => leaf.id === selectedId)
    ? selectedId
    : (leaves[0]?.id ?? '');
  const [expanded, setExpanded] = useState<Set<string>>(() => new Set(collectFolderIds(toc)));
  // Mobile-only slide-in drawer for the topic tree; desktop keeps the
  // static side-by-side layout (md: breakpoints below).
  const [tocOpen, setTocOpen] = useState(false);

  // Reset the scroll position of the nearest scrollable ancestor (the
  // app shell's <main>) when the displayed document changes, so long
  // pages start from the top instead of wherever the reader happened to
  // be on the previous page.
  const rootRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = rootRef.current;
    const scroller = el?.closest<HTMLElement>('main');
    if (!el || !scroller) return;
    const offset = el.getBoundingClientRect().top - scroller.getBoundingClientRect().top;
    scroller.scrollTo({ top: scroller.scrollTop + offset });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [effectiveId, locale]);

  const selectedIndex = leaves.findIndex((leaf) => leaf.id === effectiveId);
  const selected = selectedIndex >= 0 ? leaves[selectedIndex] : undefined;
  const path = useMemo(
    () => (effectiveId ? findHelpPath(toc, effectiveId) : null),
    [toc, effectiveId],
  );
  // Locale the displayed document actually came from — relative image paths
  // inside it resolve against this locale (fallback docs use fallback art).
  const doc = useMemo(
    () => resolveHelpDoc(locale, selected?.file ?? ''),
    [locale, selected?.file],
  );

  const handleSelect = (id: string, ancestors: string[]) => {
    setSelectedId(id);
    setTocOpen(false);
    setExpanded((prev) => {
      const next = new Set(prev);
      for (const folderId of ancestors) next.add(folderId);
      return next;
    });
  };

  const handleToggle = (id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const prev = selectedIndex > 0 ? leaves[selectedIndex - 1] : undefined;
  const next = selectedIndex >= 0 && selectedIndex < leaves.length - 1 ? leaves[selectedIndex + 1] : undefined;

  return (
    <div ref={rootRef}>
      <SectionHeader title={t('help.title')} description={t('help.description')}>
        <button
          type="button"
          onClick={() => setTocOpen(true)}
          className="md:hidden flex items-center gap-1.5 px-3 py-1.5 rounded-standard text-xs font-medium text-deep-olive hover:text-warm-black hover:bg-pale-sand border border-golden-sand transition-colors duration-vt-base"
        >
          <ListTree className="w-3.5 h-3.5 shrink-0" />
          {t('help.toc')}
        </button>
      </SectionHeader>
      {tocOpen && (
        <div
          className="fixed inset-0 z-30 bg-warm-black/50 md:hidden"
          onClick={() => setTocOpen(false)}
        />
      )}
      <div className="flex flex-col md:flex-row gap-6 items-start">
        <nav
          aria-label={t('help.title')}
          className={cn(
            'fixed inset-y-0 left-0 z-40 w-72 max-w-[80vw] p-4 overflow-y-auto vt-scrollbar',
            'bg-warm-ivory border-r border-golden-sand transition-transform duration-200',
            'md:static md:z-auto md:translate-x-0 md:w-60 md:shrink-0 md:p-2 md:sticky md:top-0',
            'md:max-h-[70vh] md:rounded-comfortable md:border md:border-golden-sand',
            tocOpen ? 'translate-x-0' : '-translate-x-full',
          )}
        >
          <div className="flex items-center justify-between mb-2 md:hidden">
            <span className="text-sm font-semibold text-warm-black">{t('help.title')}</span>
            <button
              type="button"
              onClick={() => setTocOpen(false)}
              className="p-1 rounded-tight text-warm-khaki hover:text-warm-black transition-colors duration-vt-base"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
          <div className="space-y-0.5">
            {toc.map((node) => (
              <HelpTreeRow
                key={node.id}
                node={node}
                depth={0}
                ancestors={[]}
                selectedId={effectiveId}
                expanded={expanded}
                onSelect={handleSelect}
                onToggle={handleToggle}
              />
            ))}
          </div>
        </nav>
        <article className="flex-1 min-w-0 w-full rounded-comfortable border border-golden-sand bg-warm-ivory p-6">
          {selected && path ? (
            <>
              <p className="text-2xs text-warm-khaki mb-1">
                {path
                  .slice(0, -1)
                  .map((node) => node.title)
                  .join(' / ')}
              </p>
              <h3 className="text-base font-semibold text-warm-black mb-3">{selected.title}</h3>
              {effectiveId === LICENSES_DOC_ID ? (
                <LicensePanel />
              ) : (
                <HelpMarkdown
                  markdown={doc.content}
                  docLocale={doc.locale}
                />
              )}
              <div className="flex items-center justify-between gap-3 mt-6 pt-4 border-t border-golden-sand">
                {prev ? (
                  <button
                    type="button"
                    onClick={() => handleSelect(prev.id, findHelpPath(toc, prev.id)?.slice(0, -1).map((n) => n.id) ?? [])}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-standard text-xs font-medium text-deep-olive hover:text-warm-black hover:bg-pale-sand transition-colors duration-vt-base"
                  >
                    <ChevronLeft className="w-3.5 h-3.5 shrink-0" />
                    <span className="truncate">{prev.title}</span>
                  </button>
                ) : (
                  <span />
                )}
                {next ? (
                  <button
                    type="button"
                    onClick={() => handleSelect(next.id, findHelpPath(toc, next.id)?.slice(0, -1).map((n) => n.id) ?? [])}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-standard text-xs font-medium text-deep-olive hover:text-warm-black hover:bg-pale-sand transition-colors duration-vt-base"
                  >
                    <span className="truncate">{next.title}</span>
                    <ChevronRight className="w-3.5 h-3.5 shrink-0" />
                  </button>
                ) : (
                  <span />
                )}
              </div>
            </>
          ) : (
            <p className="text-sm text-warm-khaki">{t('help.empty')}</p>
          )}
        </article>
      </div>
    </div>
  );
}
