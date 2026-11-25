import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  AlertCircle,
  Boxes,
  ChevronDown,
  ChevronUp,
  Copy,
  Eye,
  EyeOff,
  Folder,
  FolderOpen,
  FolderPlus,
  Loader2,
  Pencil,
  Plus,
  Trash2,
  X,
} from 'lucide-react';
import {
  createNode,
  duplicateNode,
  getEnvironmentConfig,
  moveNode,
  removeNode,
  renameNode,
  updateLeafVariables,
} from '@/api';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { ContextMenu, type ContextMenuItem } from '@/components/shared/ContextMenu';
import { SecretField } from '@/components/shared/SecretField';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { useToast } from '@/components/shared/Toast';
import { useConfigSection } from '@/hooks/use-config-section';
import { useTranslation } from '@/i18n';
import type {
  EnvironmentConfig,
  EnvironmentGroup,
  EnvironmentLeaf,
  EnvironmentNode,
  EnvironmentVariable,
} from '@/types';

const inputClassName = [
  'w-full bg-bg-inset px-2 py-1.5 text-sm font-normal',
  'rounded-standard border border-golden-sand',
  'transition-colors duration-vt-base',
  'focus:outline-none focus:border-amber-gold',
  'disabled:opacity-50 disabled:cursor-not-allowed',
  'text-warm-black',
].join(' ');

const ROOT_ID = '__root__';

/** Server-managed leaves (the MCP bearer token) are displayed read-only. */
function isNodeReadOnly(node: EnvironmentNode): boolean {
  return node.kind === 'leaf' && node.readonly === true;
}

// ---------------------------------------------------------------------------
// Tree helpers (pure)
// ---------------------------------------------------------------------------

interface FormVariable {
  key: string;
  value: string;
}

/** Find a descendant node by id anywhere in the subtree (root excluded). */
function findNode(group: EnvironmentGroup, id: string): EnvironmentNode | null {
  for (const child of group.children) {
    if (child.id === id) return child;
    if (child.kind === 'group') {
      const found = findNode(child, id);
      if (found) return found;
    }
  }
  return null;
}

/** Is `descendantId` anywhere inside the subtree rooted at `ancestor`? */
function isDescendant(ancestor: EnvironmentNode, descendantId: string): boolean {
  if (ancestor.id === descendantId) return true;
  if (ancestor.kind !== 'group') return false;
  return ancestor.children.some((c) => isDescendant(c, descendantId));
}

/** Count Leaf nodes across the subtree rooted at `group`. */
function countLeaves(group: EnvironmentGroup): number {
  let count = 0;
  const walk = (node: EnvironmentNode) => {
    if (node.kind === 'leaf') count += 1;
    else node.children.forEach(walk);
  };
  group.children.forEach(walk);
  return count;
}

/** Count all descendant nodes (groups + leaves) under `group`. */
function countDescendants(group: EnvironmentGroup): number {
  let count = 0;
  const walk = (node: EnvironmentNode) => {
    count += 1;
    if (node.kind === 'group') node.children.forEach(walk);
  };
  group.children.forEach(walk);
  return count;
}

/** The path of names from the root down to `id`, for display in confirmations. */
function namePath(root: EnvironmentGroup, id: string): string {
  const path: string[] = [];
  const walk = (group: EnvironmentGroup): boolean => {
    for (const child of group.children) {
      if (child.id === id) {
        path.push(child.name);
        return true;
      }
      if (child.kind === 'group') {
        path.push(child.name);
        if (walk(child)) return true;
        path.pop();
      }
    }
    return false;
  };
  walk(root);
  return path.join(' / ');
}

// ---------------------------------------------------------------------------
// Inline create state
// ---------------------------------------------------------------------------

interface InlineCreate {
  parentId: string | null; // null = under root
  kind: 'group' | 'leaf';
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function EnvironmentSection() {
  const { addToast } = useToast();
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const [revealedLeaves, setRevealedLeaves] = useState<Set<string>>(new Set());
  const [selectedLeafId, setSelectedLeafId] = useState<string | null>(null);
  const [removeTarget, setRemoveTarget] = useState<EnvironmentNode | null>(null);
  const [inlineCreate, setInlineCreate] = useState<InlineCreate | null>(null);
  const [inlineName, setInlineName] = useState('');
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState('');
  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    node: EnvironmentNode | null; // null = background/root
  } | null>(null);
  const [dragId, setDragId] = useState<string | null>(null);
  const [dropTarget, setDropTarget] = useState<string | null>(null); // group id or ROOT_ID
  const [submitting, setSubmitting] = useState(false);
  // Synchronous re-entry guard for the inline create/rename submits: their
  // inputs fire onBlur when they become disabled (or unmount) mid-submit,
  // which would otherwise invoke confirm a second time and create/rename
  // the node twice.
  const inlineSubmitInFlight = useRef(false);

  // Leaf variable editor state (mirrors the selected leaf's variables).
  const [leafVars, setLeafVars] = useState<FormVariable[]>([]);

  const fetchFn = useCallback(() => getEnvironmentConfig(), []);
  const saveFn = useCallback((config: EnvironmentConfig) => Promise.resolve(config), []);
  const { data, loading, saving, loadError, saveError, updateField } =
    useConfigSection<EnvironmentConfig>(fetchFn, saveFn);

  const root = data?.root;

  const selectedLeaf = useMemo(() => {
    if (!root || !selectedLeafId) return null;
    const node = findNode(root, selectedLeafId);
    return node && node.kind === 'leaf' ? node : null;
  }, [root, selectedLeafId]);

  // Default-expand every group on first load.
  useEffect(() => {
    if (!root) return;
    setExpanded((prev) => {
      if (prev.size > 0) return prev;
      const next = new Set<string>();
      const walk = (group: EnvironmentGroup) => {
        for (const child of group.children) {
          if (child.kind === 'group') {
            next.add(child.id);
            walk(child);
          }
        }
      };
      walk(root);
      return next;
    });
  }, [root]);

  // Sync leaf editor state when selection changes.
  useEffect(() => {
    if (selectedLeaf) {
      setLeafVars(selectedLeaf.variables.map((v) => ({ ...v })));
    } else {
      setLeafVars([]);
    }
  }, [selectedLeafId, selectedLeaf]);

  const leafReadOnly = selectedLeaf?.readonly === true;

  const leafCount = useMemo(() => (root ? countLeaves(root) : 0), [root]);

  const refresh = useCallback(async () => {
    try {
      const fresh = await getEnvironmentConfig();
      updateField('root', fresh.root);
    } catch (err) {
      addToast('error', err instanceof Error ? err.message : t('environment.loadError'));
    }
  }, [addToast, t, updateField]);

  // --- expansion helpers --------------------------------------------------
  const toggleExpanded = useCallback((id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const expandAll = useCallback(() => {
    if (!root) return;
    const next = new Set<string>();
    const walk = (group: EnvironmentGroup) => {
      for (const child of group.children) {
        if (child.kind === 'group') {
          next.add(child.id);
          walk(child);
        }
      }
    };
    walk(root);
    setExpanded(next);
  }, [root]);

  const collapseAll = useCallback(() => setExpanded(new Set()), []);

  const toggleRevealed = useCallback((id: string) => {
    setRevealedLeaves((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  // --- create / rename ----------------------------------------------------
  const beginCreate = useCallback((parentId: string | null, kind: 'group' | 'leaf') => {
    setInlineCreate({ parentId, kind });
    setInlineName('');
    setRenamingId(null);
  }, []);

  const cancelInlineCreate = useCallback(() => {
    setInlineCreate(null);
    setInlineName('');
  }, []);

  const submitInlineCreate = useCallback(async () => {
    if (!inlineCreate || inlineSubmitInFlight.current) return;
    inlineSubmitInFlight.current = true;
    const name = inlineName.trim();
    if (!name) {
      inlineSubmitInFlight.current = false;
      addToast('error', t('environment.nameRequired'));
      return;
    }
    setSubmitting(true);
    try {
      const created = await createNode({ parentId: inlineCreate.parentId, kind: inlineCreate.kind, name });
      await refresh();
      // Auto-expand the parent (if a group) and select the new node.
      const parentId = inlineCreate.parentId;
      if (parentId) setExpanded((prev) => new Set(prev).add(parentId));
      if (created.kind === 'leaf') setSelectedLeafId(created.id);
      addToast('success', t('environment.created', { name }));
      cancelInlineCreate();
    } catch (err) {
      addToast('error', err instanceof Error ? err.message : t('environment.loadError'));
    } finally {
      inlineSubmitInFlight.current = false;
      setSubmitting(false);
    }
  }, [addToast, cancelInlineCreate, inlineCreate, inlineName, refresh, t]);

  const beginRename = useCallback((node: EnvironmentNode) => {
    setRenamingId(node.id);
    setRenameValue(node.name);
    setInlineCreate(null);
  }, []);

  const cancelRename = useCallback(() => {
    setRenamingId(null);
    setRenameValue('');
  }, []);

  const submitRename = useCallback(
    async (id: string) => {
      if (inlineSubmitInFlight.current) return;
      const name = renameValue.trim();
      if (!name) {
        addToast('error', t('environment.nameRequired'));
        return;
      }
      inlineSubmitInFlight.current = true;
      setSubmitting(true);
      try {
        await renameNode(id, { name });
        await refresh();
        addToast('success', t('environment.renamed', { name }));
        cancelRename();
      } catch (err) {
        addToast('error', err instanceof Error ? err.message : t('environment.loadError'));
      } finally {
        inlineSubmitInFlight.current = false;
        setSubmitting(false);
      }
    },
    [addToast, cancelRename, refresh, renameValue, t],
  );

  // --- duplicate / delete / move -----------------------------------------
  const handleDuplicate = useCallback(
    async (node: EnvironmentNode) => {
      setSubmitting(true);
      try {
        await duplicateNode(node.id);
        await refresh();
        addToast('success', t('environment.duplicated', { name: node.name }));
      } catch (err) {
        addToast('error', err instanceof Error ? err.message : t('environment.loadError'));
      } finally {
        setSubmitting(false);
      }
    },
    [addToast, refresh, t],
  );

  const handleConfirmRemove = useCallback(async () => {
    if (!removeTarget) return;
    const target = removeTarget;
    setSubmitting(true);
    try {
      await removeNode(target.id);
      await refresh();
      if (selectedLeafId === target.id) setSelectedLeafId(null);
      addToast('success', t('environment.removed'));
    } catch (err) {
      addToast('error', err instanceof Error ? err.message : t('environment.loadError'));
    } finally {
      setSubmitting(false);
      setRemoveTarget(null);
    }
  }, [addToast, refresh, removeTarget, selectedLeafId, t]);

  const handleMove = useCallback(
    async (nodeId: string, targetParentId: string | null) => {
      if (!root) return;
      // Validate client-side to avoid a round-trip for obvious errors.
      const node = findNode(root, nodeId);
      if (!node) return;
      if (targetParentId === nodeId) return;
      if (targetParentId && isDescendant(node, targetParentId)) {
        addToast('error', t('environment.invalidDropTarget'));
        return;
      }
      setSubmitting(true);
      try {
        await moveNode(nodeId, { targetParentId, index: null });
        await refresh();
      } catch (err) {
        addToast('error', err instanceof Error ? err.message : t('environment.loadError'));
      } finally {
        setSubmitting(false);
        setDragId(null);
        setDropTarget(null);
      }
    },
    [addToast, refresh, root, t],
  );

  // --- leaf variables -----------------------------------------------------
  const handleAddVariable = useCallback(() => {
    setLeafVars((current) => [...current, { key: '', value: '' }]);
  }, []);

  const handleRemoveVariable = useCallback((index: number) => {
    setLeafVars((current) => current.filter((_, i) => i !== index));
  }, []);

  const handleUpdateVariable = useCallback(
    (index: number, field: 'key' | 'value', val: string) => {
      setLeafVars((current) =>
        current.map((v, i) => (i === index ? { ...v, [field]: val } : v)),
      );
    },
    [],
  );

  const handleSaveLeafVariables = useCallback(async () => {
    if (!selectedLeaf) return;
    setSubmitting(true);
    const variables: EnvironmentVariable[] = leafVars
      .filter((v) => v.key.trim())
      .map((v) => ({ key: v.key.trim(), value: v.value }));
    try {
      await updateLeafVariables(selectedLeaf.id, { variables });
      await refresh();
      addToast('success', t('environment.variablesSaved', { name: selectedLeaf.name }));
    } catch (err) {
      addToast('error', err instanceof Error ? err.message : t('environment.loadError'));
    } finally {
      setSubmitting(false);
    }
  }, [addToast, leafVars, refresh, selectedLeaf, t]);

  // --- context menu items -------------------------------------------------
  const openContextMenu = useCallback(
    (e: React.MouseEvent, node: EnvironmentNode | null) => {
      e.preventDefault();
      setContextMenu({ x: e.clientX, y: e.clientY, node });
    },
    [],
  );

  const closeContextMenu = useCallback(() => setContextMenu(null), []);

  const menuItems = useMemo<ContextMenuItem[]>(() => {
    if (!contextMenu) return [];
    const node = contextMenu.node;
    if (!node) {
      // Background / root: only creation actions.
      return [
        {
          id: 'new-group',
          label: t('environment.newGroup'),
          icon: <FolderPlus className="w-3.5 h-3.5" />,
          onClick: () => beginCreate(null, 'group'),
        },
        {
          id: 'new-leaf',
          label: t('environment.newLeaf'),
          icon: <Boxes className="w-3.5 h-3.5" />,
          onClick: () => beginCreate(null, 'leaf'),
        },
      ];
    }
    // Server-managed leaves offer no actions: they are generated from the
    // MCP settings and cannot be renamed, duplicated, moved or removed.
    if (isNodeReadOnly(node)) return [];
    const items: ContextMenuItem[] = [];
    if (node.kind === 'group') {
      items.push(
        {
          id: 'add-child-group',
          label: t('environment.newGroupHere'),
          icon: <FolderPlus className="w-3.5 h-3.5" />,
          onClick: () => beginCreate(node.id, 'group'),
        },
        {
          id: 'add-child-leaf',
          label: t('environment.newLeafHere'),
          icon: <Boxes className="w-3.5 h-3.5" />,
          onClick: () => beginCreate(node.id, 'leaf'),
        },
      );
    }
    items.push(
      {
        id: 'rename',
        label: t('environment.rename'),
        icon: <Pencil className="w-3.5 h-3.5" />,
        onClick: () => beginRename(node),
      },
      {
        id: 'duplicate',
        label: t('environment.duplicate'),
        icon: <Copy className="w-3.5 h-3.5" />,
        onClick: () => handleDuplicate(node),
      },
      {
        id: 'remove',
        label: t('common.remove'),
        icon: <Trash2 className="w-3.5 h-3.5" />,
        danger: true,
        separatorBefore: true,
        onClick: () => setRemoveTarget(node),
      },
    );
    return items;
  }, [beginCreate, beginRename, contextMenu, handleDuplicate, t]);

  const removeMessage = useMemo(() => {
    if (!removeTarget || !root) return '';
    const path = namePath(root, removeTarget.id);
    if (removeTarget.kind === 'leaf') {
      const vars = removeTarget.variables.length;
      return t('environment.removeLeafMessage', { name: path, count: vars });
    }
    const groups = removeTarget.children.length;
    const total = countDescendants(removeTarget) - 1; // exclude the node itself
    return t('environment.removeGroupMessage', {
      name: path,
      groupCount: groups,
      count: Math.max(0, total),
    });
  }, [removeTarget, root, t]);

  // -------------------------------------------------------------------------
  // Render
  // -------------------------------------------------------------------------

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-6 h-6 animate-spin text-warm-black" />
      </div>
    );
  }

  if (loadError || !data || !root) {
    return (
      <div className="flex items-center gap-2 text-warm-black py-8">
        <AlertCircle className="w-5 h-5" />
        <span className="font-light">{loadError ?? t('environment.loadError')}</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SectionHeader title={t('environment.title')} description={t('environment.description')} />

      {/* Tree */}
      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <div className="flex items-center justify-between gap-4 flex-wrap">
          <h3 className="text-sm font-semibold text-warm-black tracking-wide">
            {t('environment.leafCount', { count: leafCount })}
          </h3>
          {root.children.length > 0 && (
            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={expandAll}
                className="inline-flex items-center gap-1.5 px-3 py-1 text-[11px] font-medium text-warm-black hover:bg-pale-sand rounded-standard border border-golden-sand transition-colors duration-vt-base"
              >
                <ChevronDown className="w-3 h-3" />
                {t('environment.expandAll')}
              </button>
              <button
                type="button"
                onClick={collapseAll}
                className="inline-flex items-center gap-1.5 px-3 py-1 text-[11px] font-medium text-warm-black hover:bg-pale-sand rounded-standard border border-golden-sand transition-colors duration-vt-base"
              >
                <ChevronUp className="w-3 h-3" />
                {t('environment.collapseAll')}
              </button>
            </div>
          )}
        </div>

        {/* Drop zone for the whole tree (root level). */}
        <div
          onContextMenu={(e) => openContextMenu(e, null)}
          onDragOver={(e) => {
            if (!dragId) return;
            e.preventDefault();
            setDropTarget(ROOT_ID);
          }}
          onDragLeave={(e) => {
            // Only clear if leaving the container itself, not a child.
            if (e.currentTarget === e.target) setDropTarget((t2) => (t2 === ROOT_ID ? null : t2));
          }}
          onDrop={(e) => {
            if (!dragId) return;
            // Dropping on the root container moves the node to the top level.
            e.preventDefault();
            void handleMove(dragId, null);
          }}
          className={`rounded-comfortable border transition-colors duration-vt-base ${
            dropTarget === ROOT_ID
              ? 'border-amber-gold bg-pale-sand/60'
              : 'border-dashed border-golden-sand'
          }`}
        >
          {root.children.length === 0 && !inlineCreate ? (
            <div className="p-4 text-xs text-deep-olive">{t('environment.emptyTree')}</div>
          ) : (
            <div className="p-2 space-y-1">
              {inlineCreate && inlineCreate.parentId === null && (
                <InlineCreateRow
                  kind={inlineCreate.kind}
                  value={inlineName}
                  onChange={setInlineName}
                  submitting={submitting}
                  onConfirm={submitInlineCreate}
                  onCancel={cancelInlineCreate}
                />
              )}
              {root.children.map((node) => (
                <TreeNode
                  key={node.id}
                  node={node}
                  depth={0}
                  expanded={expanded}
                  revealedLeaves={revealedLeaves}
                  selectedLeafId={selectedLeafId}
                  renamingId={renamingId}
                  renameValue={renameValue}
                  inlineCreate={inlineCreate}
                  inlineName={inlineName}
                  dragId={dragId}
                  dropTarget={dropTarget}
                  submitting={submitting}
                  onToggleExpand={toggleExpanded}
                  onToggleReveal={toggleRevealed}
                  onSelectLeaf={setSelectedLeafId}
                  onAddChild={beginCreate}
                  onEditVariables={(n) => {
                    setSelectedLeafId(n.id);
                  }}
                  onRename={beginRename}
                  onRenameChange={setRenameValue}
                  onRenameConfirm={submitRename}
                  onRenameCancel={cancelRename}
                  onDuplicate={handleDuplicate}
                  onRemove={setRemoveTarget}
                  onContextMenu={openContextMenu}
                  onInlineNameChange={setInlineName}
                  onInlineConfirm={submitInlineCreate}
                  onInlineCancel={cancelInlineCreate}
                  onDragStart={(id) => setDragId(id)}
                  onDragEnd={() => {
                    setDragId(null);
                    setDropTarget(null);
                  }}
                  onDrop={(parentId) => handleMove(dragId ?? '', parentId)}
                  onDropTargetChange={setDropTarget}
                />
              ))}
            </div>
          )}
        </div>

        <div className="flex items-center gap-2 flex-wrap">
          <button
            type="button"
            onClick={() => beginCreate(null, 'group')}
            disabled={submitting}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-[11px] font-medium text-warm-black hover:bg-pale-sand rounded-standard border border-golden-sand transition-colors duration-vt-base disabled:opacity-50"
          >
            <FolderPlus className="w-3.5 h-3.5" />
            {t('environment.newGroup')}
          </button>
          <button
            type="button"
            onClick={() => beginCreate(null, 'leaf')}
            disabled={submitting}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 text-[11px] font-medium text-warm-black hover:bg-pale-sand rounded-standard border border-golden-sand transition-colors duration-vt-base disabled:opacity-50"
          >
            <Boxes className="w-3.5 h-3.5" />
            {t('environment.newLeaf')}
          </button>
        </div>
      </div>

      {/* Leaf variable editor */}
      {selectedLeaf && (
        <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
          <div className="flex items-center justify-between gap-4">
            <h3 className="text-sm font-semibold text-warm-black tracking-wide">
              {t('environment.editVariables', { name: selectedLeaf.name })}
            </h3>
            <button
              type="button"
              onClick={() => setSelectedLeafId(null)}
              className="inline-flex items-center justify-center w-7 h-7 rounded-standard text-warm-khaki hover:text-warm-black hover:bg-pale-sand transition-colors duration-vt-base"
              aria-label={t('common.cancel')}
            >
              <X className="w-4 h-4" />
            </button>
          </div>

          {leafReadOnly && (
            <p className="text-xs text-deep-olive leading-relaxed">
              {t('environment.readOnlyHint')}
            </p>
          )}

          <div className="space-y-3">
            {leafVars.length === 0 ? (
              <div className="text-xs text-deep-olive py-2">{t('environment.noVariables')}</div>
            ) : (
              <div className="space-y-3">
                {leafVars.map((variable, index) => (
                  <div
                    key={index}
                    className="rounded-comfortable border border-golden-sand p-3 bg-pale-sand/30 space-y-3"
                  >
                    <div className="flex items-center justify-between gap-3">
                      <label className="text-[11px] font-medium text-deep-olive uppercase tracking-wide">
                        {t('environment.varKeyLabel')} {index + 1}
                      </label>
                      {!leafReadOnly && (
                        <button
                          type="button"
                          onClick={() => handleRemoveVariable(index)}
                          className="inline-flex items-center justify-center w-6 h-6 shrink-0 rounded-standard text-warm-khaki hover:text-vt-error transition-colors duration-vt-base"
                          aria-label={t('common.remove')}
                        >
                          <X className="w-3.5 h-3.5" />
                        </button>
                      )}
                    </div>
                    <input
                      type="text"
                      value={variable.key}
                      onChange={(e) => handleUpdateVariable(index, 'key', e.target.value)}
                      readOnly={leafReadOnly}
                      className={inputClassName}
                      placeholder="API_KEY"
                    />
                    <SecretField
                      value={variable.value}
                      onChange={(e) => handleUpdateVariable(index, 'value', e.target.value)}
                      // `readOnly` (not `disabled`) keeps the reveal eye
                      // working on server-managed leaves while blocking edits.
                      readOnly={leafReadOnly}
                      className={`${inputClassName} pr-10`}
                      placeholder={t('environment.varValuePlaceholder')}
                    />
                  </div>
                ))}
              </div>
            )}
            {!leafReadOnly && (
              <button
                type="button"
                onClick={handleAddVariable}
                className="inline-flex items-center gap-1.5 px-3 py-1 text-[11px] font-medium text-warm-black hover:bg-pale-sand rounded-standard border border-golden-sand transition-colors duration-vt-base"
              >
                <Plus className="w-3 h-3" />
                {t('environment.addVariable')}
              </button>
            )}
          </div>

          {!leafReadOnly && (
            <div className="flex flex-wrap items-center gap-3">
              <button
                type="button"
                onClick={handleSaveLeafVariables}
                disabled={submitting}
                className="inline-flex items-center gap-2 px-4 py-1.5 bg-amber-gold text-text-inverse rounded-standard font-medium text-xs hover:opacity-90 transition-opacity border border-amber-gold disabled:opacity-50"
              >
                {submitting ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <Plus className="w-4 h-4" />
                )}
                {t('environment.saveVariables')}
              </button>
            </div>
          )}
        </div>
      )}

      <ConfirmDialog
        open={removeTarget !== null}
        title={t('environment.removeTitle')}
        message={removeMessage}
        confirmLabel={t('common.remove')}
        variant="danger"
        onConfirm={handleConfirmRemove}
        onCancel={() => setRemoveTarget(null)}
      />

      <ContextMenu
        open={contextMenu !== null}
        x={contextMenu?.x ?? 0}
        y={contextMenu?.y ?? 0}
        items={menuItems}
        onClose={closeContextMenu}
      />

      {(saving || saveError) && (
        <div className={`text-xs ${saveError ? 'text-vt-error' : 'text-warm-khaki'}`}>
          {saveError ?? t('common.saving')}
        </div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Inline create row
// ---------------------------------------------------------------------------

interface InlineCreateRowProps {
  kind: 'group' | 'leaf';
  value: string;
  onChange: (v: string) => void;
  submitting: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

function InlineCreateRow({
  kind,
  value,
  onChange,
  submitting,
  onConfirm,
  onCancel,
}: InlineCreateRowProps) {
  const { t } = useTranslation();
  return (
    <div className="flex items-center gap-2 py-1 px-1">
      {kind === 'group' ? (
        <Folder className="w-4 h-4 text-amber-gold shrink-0" />
      ) : (
        <Boxes className="w-4 h-4 text-amber-gold shrink-0" />
      )}
      <input
        type="text"
        autoFocus
        value={value}
        disabled={submitting}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            onConfirm();
          } else if (e.key === 'Escape') {
            e.preventDefault();
            onCancel();
          }
        }}
        onBlur={() => {
          // Cancel on blur if empty; otherwise confirm.
          if (value.trim()) onConfirm();
          else onCancel();
        }}
        className={inputClassName}
        placeholder={t('environment.namePlaceholder')}
      />
    </div>
  );
}

// ---------------------------------------------------------------------------
// Recursive tree node
// ---------------------------------------------------------------------------

interface TreeNodeProps {
  node: EnvironmentNode;
  depth: number;
  expanded: Set<string>;
  revealedLeaves: Set<string>;
  selectedLeafId: string | null;
  renamingId: string | null;
  renameValue: string;
  inlineCreate: InlineCreate | null;
  inlineName: string;
  dragId: string | null;
  dropTarget: string | null;
  submitting: boolean;
  onToggleExpand: (id: string) => void;
  onToggleReveal: (id: string) => void;
  onSelectLeaf: (id: string | null) => void;
  onAddChild: (parentId: string, kind: 'group' | 'leaf') => void;
  onEditVariables: (node: EnvironmentLeaf) => void;
  onRename: (node: EnvironmentNode) => void;
  onRenameChange: (v: string) => void;
  onRenameConfirm: (id: string) => void;
  onRenameCancel: () => void;
  onDuplicate: (node: EnvironmentNode) => void;
  onRemove: (node: EnvironmentNode) => void;
  onContextMenu: (e: React.MouseEvent, node: EnvironmentNode) => void;
  onInlineNameChange: (v: string) => void;
  onInlineConfirm: () => void;
  onInlineCancel: () => void;
  onDragStart: (id: string) => void;
  onDragEnd: () => void;
  onDrop: (targetParentId: string | null) => void;
  onDropTargetChange: (target: string | null) => void;
}

function TreeNode(props: TreeNodeProps) {
  const {
    node,
    depth,
    expanded,
    revealedLeaves,
    selectedLeafId,
    renamingId,
    renameValue,
    inlineCreate,
    inlineName,
    dragId,
    dropTarget,
    submitting,
    onToggleExpand,
    onToggleReveal,
    onSelectLeaf,
    onAddChild,
    onEditVariables,
    onRename,
    onRenameChange,
    onRenameConfirm,
    onRenameCancel,
    onDuplicate,
    onRemove,
    onContextMenu,
    onInlineNameChange,
    onInlineConfirm,
    onInlineCancel,
    onDragStart,
    onDragEnd,
    onDrop,
    onDropTargetChange,
  } = props;
  const { t } = useTranslation();

  const isGroupNode = node.kind === 'group';
  const isOpen = expanded.has(node.id);
  const revealed = revealedLeaves.has(node.id);
  const isSelected = selectedLeafId === node.id;
  const isRenaming = renamingId === node.id;
  const isDropTarget = dropTarget === node.id;
  // Server-managed leaves (the MCP bearer token) are not user-editable:
  // no rename/duplicate/remove, no dragging.
  const isReadOnly = isNodeReadOnly(node);

  // A node is a valid drop target only if it's a group and not the dragged
  // node itself or one of its descendants (the latter is checked in handleMove).
  const canBeDropTarget = isGroupNode && dragId !== null && dragId !== node.id;

  const meta = isGroupNode
    ? `${node.children.length} ${t('environment.childrenLabel')}`
    : `${node.variables.length} ${t('environment.variableCount', { count: node.variables.length })}`;

  return (
    <div>
      <div
        draggable={!isRenaming && !isReadOnly}
        onDragStart={(e) => {
          // Leaf or group: both are draggable.
          e.dataTransfer.effectAllowed = 'move';
          e.dataTransfer.setData('text/plain', node.id);
          onDragStart(node.id);
        }}
        onDragEnd={onDragEnd}
        onDragOver={(e) => {
          if (!canBeDropTarget) return;
          e.preventDefault();
          if (isDropTarget) return;
          onDropTargetChange(node.id);
        }}
        onDragLeave={(e) => {
          if (e.currentTarget === e.target && isDropTarget) onDropTargetChange(null);
        }}
        onDrop={(e) => {
          if (!canBeDropTarget) return;
          e.preventDefault();
          e.stopPropagation();
          onDrop(node.id);
        }}
        onContextMenu={(e) => onContextMenu(e, node)}
        className={`group flex items-center gap-2 rounded-standard py-1.5 px-2 transition-colors duration-vt-base ${
          isDropTarget ? 'bg-pale-sand/70 ring-1 ring-amber-gold' : 'hover:bg-pale-sand/40'
        } ${isSelected ? 'bg-pale-sand/60' : ''}`}
        style={{ marginLeft: depth * 14 }}
      >
        {isGroupNode ? (
          <button
            type="button"
            onClick={() => onToggleExpand(node.id)}
            className="inline-flex items-center justify-center w-5 h-5 shrink-0 rounded-tight text-warm-black hover:bg-golden-cream transition-colors duration-vt-base"
            aria-label={isOpen ? t('environment.collapse') : t('environment.expand')}
          >
            {isOpen ? (
              <ChevronDown className="w-4 h-4" />
            ) : (
              <ChevronUp className="w-4 h-4" />
            )}
          </button>
        ) : (
          <span className="inline-flex items-center justify-center w-5 h-5 shrink-0" />
        )}

        <span className="shrink-0 text-amber-gold">
          {isGroupNode ? (
            isOpen ? (
              <FolderOpen className="w-4 h-4" />
            ) : (
              <Folder className="w-4 h-4" />
            )
          ) : (
            <Boxes className="w-4 h-4" />
          )}
        </span>

        {isRenaming ? (
          <input
            type="text"
            autoFocus
            value={renameValue}
            disabled={submitting}
            onChange={(e) => onRenameChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                onRenameConfirm(node.id);
              } else if (e.key === 'Escape') {
                e.preventDefault();
                onRenameCancel();
              }
            }}
            onBlur={() => {
              if (renameValue.trim()) onRenameConfirm(node.id);
              else onRenameCancel();
            }}
            className={`${inputClassName} h-7 py-0.5`}
          />
        ) : (
          <button
            type="button"
            onClick={() => {
              if (node.kind === 'leaf') {
                onSelectLeaf(node.id);
                onEditVariables(node);
              } else {
                // Clicking a group clears any selected leaf so the variable
                // editor below does not keep showing the previous collection.
                onSelectLeaf(null);
                onToggleExpand(node.id);
              }
            }}
            className="flex-1 text-left min-w-0"
          >
            <span className="text-sm font-medium text-warm-black truncate">{node.name}</span>
            <span className="ml-2 text-[11px] text-deep-olive">{meta}</span>
          </button>
        )}

        {/* Hover actions (quick access; full set in context menu). */}
        {!isRenaming && (
          <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-vt-base shrink-0">
            {node.kind === 'leaf' && node.variables.length > 0 && (
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  onToggleReveal(node.id);
                }}
                className="inline-flex items-center justify-center w-7 h-7 rounded-standard text-warm-black hover:bg-golden-cream transition-colors duration-vt-base"
                title={revealed ? t('environment.hideValues') : t('environment.revealValues')}
                aria-label={revealed ? t('environment.hideValues') : t('environment.revealValues')}
              >
                {revealed ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
              </button>
            )}
            {isGroupNode && (
              <>
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    onAddChild(node.id, 'group');
                  }}
                  disabled={submitting}
                  className="inline-flex items-center justify-center w-7 h-7 rounded-standard text-warm-black hover:bg-golden-cream transition-colors duration-vt-base disabled:opacity-50"
                  title={t('environment.newGroupHere')}
                  aria-label={t('environment.newGroupHere')}
                >
                  <FolderPlus className="w-3.5 h-3.5" />
                </button>
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    onAddChild(node.id, 'leaf');
                  }}
                  disabled={submitting}
                  className="inline-flex items-center justify-center w-7 h-7 rounded-standard text-warm-black hover:bg-golden-cream transition-colors duration-vt-base disabled:opacity-50"
                  title={t('environment.newLeafHere')}
                  aria-label={t('environment.newLeafHere')}
                >
                  <Boxes className="w-3.5 h-3.5" />
                </button>
              </>
            )}
            {!isReadOnly && (
              <>
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    onRename(node);
                  }}
                  disabled={submitting}
                  className="inline-flex items-center justify-center w-7 h-7 rounded-standard text-warm-black hover:bg-golden-cream transition-colors duration-vt-base disabled:opacity-50"
                  title={t('environment.rename')}
                  aria-label={t('environment.rename')}
                >
                  <Pencil className="w-3.5 h-3.5" />
                </button>
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    onDuplicate(node);
                  }}
                  disabled={submitting}
                  className="inline-flex items-center justify-center w-7 h-7 rounded-standard border border-accent-text text-accent-text hover:bg-accent-text hover:text-text-inverse transition-colors duration-vt-base disabled:opacity-50"
                  title={t('environment.duplicate')}
                  aria-label={t('environment.duplicate')}
                >
                  <Copy className="w-3.5 h-3.5" />
                </button>
                <button
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    onRemove(node);
                  }}
                  disabled={submitting}
                  className="inline-flex items-center justify-center w-7 h-7 rounded-standard border border-vt-error text-vt-error hover:bg-vt-error hover:text-text-inverse transition-colors duration-vt-base disabled:opacity-50"
                  title={t('common.remove')}
                  aria-label={t('common.remove')}
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </>
            )}
          </div>
        )}
      </div>

      {/* Leaf variables inline preview when revealed. */}
      {node.kind === 'leaf' && revealed && node.variables.length > 0 && (
        <div className="py-1 pl-12 space-y-0.5">
          {node.variables.map((v, i) => (
            <div key={i} className="flex gap-2 text-xs font-mono">
              <span className="text-warm-black">{v.key}</span>
              <span className="text-warm-khaki">=</span>
              <span className="text-deep-olive break-all">{v.value || '••••••••'}</span>
            </div>
          ))}
        </div>
      )}

      {/* Children (group only). */}
      {isGroupNode && isOpen && (
        <div className="mt-0.5 space-y-0.5">
          {inlineCreate && inlineCreate.parentId === node.id && (
            <InlineCreateRow
              kind={inlineCreate.kind}
              value={inlineName}
              onChange={onInlineNameChange}
              submitting={submitting}
              onConfirm={onInlineConfirm}
              onCancel={onInlineCancel}
            />
          )}
          {node.children.map((child) => (
            <TreeNode
              key={child.id}
              node={child}
              depth={depth + 1}
              expanded={expanded}
              revealedLeaves={revealedLeaves}
              selectedLeafId={selectedLeafId}
              renamingId={renamingId}
              renameValue={renameValue}
              inlineCreate={inlineCreate}
              inlineName={inlineName}
              dragId={dragId}
              dropTarget={dropTarget}
              submitting={submitting}
              onToggleExpand={onToggleExpand}
              onToggleReveal={onToggleReveal}
              onSelectLeaf={onSelectLeaf}
              onAddChild={onAddChild}
              onEditVariables={onEditVariables}
              onRename={onRename}
              onRenameChange={onRenameChange}
              onRenameConfirm={onRenameConfirm}
              onRenameCancel={onRenameCancel}
              onDuplicate={onDuplicate}
              onRemove={onRemove}
              onContextMenu={onContextMenu}
              onInlineNameChange={onInlineNameChange}
              onInlineConfirm={onInlineConfirm}
              onInlineCancel={onInlineCancel}
              onDragStart={onDragStart}
              onDragEnd={onDragEnd}
              onDrop={onDrop}
              onDropTargetChange={onDropTargetChange}
            />
          ))}
        </div>
      )}
    </div>
  );
}
