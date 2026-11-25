/**
 * Per-locale help content types.
 *
 * Each locale owns a folder `src/help/<locale>/` with a `toc.ts` (this tree
 * structure, titles included) plus one `<doc>.md` markdown file per document
 * — see `src/help/index.ts` for how they are loaded and how fallback works.
 */
export interface HelpDocRef {
  /** Stable document id, unique across the tree (drives selection state). */
  id: string;
  /** Title shown in the tree, breadcrumb and prev/next buttons. */
  title: string;
  /** Markdown file (relative to the locale folder) holding the body. */
  file: string;
}

export interface HelpTocFolder {
  id: string;
  title: string;
  /** Folder icon name, resolved through the icon map in HelpSection. */
  icon?: string;
  children: HelpTocNode[];
}

export type HelpTocNode = HelpDocRef | HelpTocFolder;

export function isHelpFolder(node: HelpTocNode): node is HelpTocFolder {
  return 'children' in node;
}
