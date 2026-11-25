/**
 * Help content registry.
 *
 * Layout (one folder per locale, folder name = locale code):
 *
 *   src/help/<locale>/toc.ts      table of contents (titles in that language)
 *   src/help/<locale>/<doc>.md    document bodies as markdown
 *   src/help/<locale>/images/     screenshots referenced from the markdown as
 *                                 relative `images/<file>` paths
 *
 * Locales without their own folder — or missing a single document — fall
 * back to `HELP_FALLBACK_LOCALE` (`en`). Missing images fall back the same
 * way; a relative image that ships for neither locale renders as a reserved
 * placeholder box in the UI. To add a language, create the folder and
 * files; no registration needed.
 *
 * Conventions inside the markdown:
 * - `images/<file>` relative paths resolve per locale (see above).
 * - Inline `` `backticks` `` mark UI elements the reader clicks/taps
 *   (buttons, menu items, form fields); the renderer styles them as chips.
 */
import { isHelpFolder, type HelpDocRef, type HelpTocNode } from './types';

export type { HelpDocRef, HelpTocFolder, HelpTocNode } from './types';
export { isHelpFolder } from './types';

export const HELP_FALLBACK_LOCALE = 'en';

const tocModules = import.meta.glob<{ default: HelpTocNode[] }>('./*/toc.ts', {
  eager: true,
});
const docModules = import.meta.glob<string>('./*/*.md', {
  eager: true,
  query: '?raw',
  import: 'default',
});
const imageModules = import.meta.glob<string>(
  './*/images/*.{png,jpg,jpeg,webp,gif,svg,webm,mp4}',
  {
    eager: true,
    query: '?url',
    import: 'default',
  },
);

// './en/toc.ts' -> 'en'
const tocs = new Map<string, HelpTocNode[]>();
for (const [path, mod] of Object.entries(tocModules)) {
  tocs.set(path.split('/')[1]!, mod.default);
}

// './en/start-overview.md' -> 'en/start-overview.md'
const docs = new Map<string, string>();
for (const [path, content] of Object.entries(docModules)) {
  docs.set(path.slice(2), content);
}

// './en/images/x.jpg' -> 'en/images/x.jpg' (bundled, hashed asset URL)
const images = new Map<string, string>();
for (const [path, url] of Object.entries(imageModules)) {
  images.set(path.slice(2), url);
}

/** Locales that ship their own help folder. */
export function hasHelpLocale(locale: string): boolean {
  return tocs.has(locale);
}

/** Table of contents for a locale, falling back to the default locale. */
export function getHelpToc(locale: string): HelpTocNode[] {
  return tocs.get(locale) ?? tocs.get(HELP_FALLBACK_LOCALE) ?? [];
}

/** Markdown body of a document, falling back to the default locale. */
export function getHelpDoc(locale: string, file: string): string {
  return (
    docs.get(`${locale}/${file}`) ??
    docs.get(`${HELP_FALLBACK_LOCALE}/${file}`) ??
    ''
  );
}

/**
 * A document's body plus the locale it actually came from. Relative image
 * paths inside the markdown resolve against that locale (a zh-CN reader
 * looking at an `en` fallback document must get the `en` screenshots).
 */
export function resolveHelpDoc(locale: string, file: string): { locale: string; content: string } {
  const own = docs.get(`${locale}/${file}`);
  if (own !== undefined) return { locale, content: own };
  return { locale: HELP_FALLBACK_LOCALE, content: getHelpDoc(locale, file) };
}

/**
 * Bundled URL of a help image `file` (e.g. `start-connect-desktop.jpg`) for
 * `locale`, falling back to the default locale; `null` when it ships for
 * neither (the renderer reserves a placeholder box).
 */
export function getHelpImage(locale: string, file: string): string | null {
  return (
    images.get(`${locale}/images/${file}`) ??
    images.get(`${HELP_FALLBACK_LOCALE}/images/${file}`) ??
    null
  );
}

/** All selectable documents in reading order (depth-first). */
export function flattenHelpLeaves(nodes: HelpTocNode[]): HelpDocRef[] {
  return nodes.flatMap((node) =>
    isHelpFolder(node) ? flattenHelpLeaves(node.children) : [node],
  );
}

/** Collect the ids of every folder in the tree. */
export function collectFolderIds(nodes: HelpTocNode[]): string[] {
  return nodes.flatMap((node) =>
    isHelpFolder(node) ? [node.id, ...collectFolderIds(node.children)] : [],
  );
}

/** Path (root first) of nodes leading to the given id, or null if not found. */
export function findHelpPath(nodes: HelpTocNode[], id: string): HelpTocNode[] | null {
  for (const node of nodes) {
    if (node.id === id) return [node];
    if (isHelpFolder(node)) {
      const path = findHelpPath(node.children, id);
      if (path) return [node, ...path];
    }
  }
  return null;
}
