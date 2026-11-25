import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  Loader2,
  AlertCircle,
  ChevronDown,
  ChevronUp,
  Download,
  Pencil,
  Upload,
  RotateCcw,
  Save,
  Trash2,
  X,
} from 'lucide-react';
import {
  getAddons,
  getAddonDetail,
  updateAddonConfig,
  saveAddonSource,
  resetAddonSource,
  uploadAddonZip,
  deleteAddon,
  exportAddonZip,
} from '@/api/addon';
import { FormField } from '@/components/shared/FormField';
import { ToggleField } from '@/components/shared/ToggleField';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { useToast } from '@/components/shared/Toast';
import { useTranslation } from '@/i18n';
import type { AddonInfo, AddonConfigField } from '@/types';

const inputClassName =
  'w-full px-2 py-1.5 bg-bg-inset border border-golden-sand rounded-standard text-warm-black text-sm placeholder:text-warm-khaki focus:outline-none focus:border-amber-gold disabled:opacity-50';

const textareaClassName =
  'w-full px-2 py-1.5 bg-bg-inset border border-golden-sand rounded-standard text-warm-black text-sm placeholder:text-warm-khaki focus:outline-none focus:border-amber-gold disabled:opacity-50 resize-y min-h-[80px]';

/**
 * Addon display groups. The backend stores the raw `addonType` (e.g. `asr`,
 * `ddns`); these groups collapse the raw spellings into the buckets the
 * UI shows. To add a future group, append an entry here — the section and
 * its "Other" fallback pick it up with no other changes.
 */
interface AddonGroup {
  id: string;
  labelKey: string;
  types: string[];
}

const ADDON_GROUPS: AddonGroup[] = [
  { id: 'app', labelKey: 'addons.groupApp', types: ['app'] },
  { id: 'llm', labelKey: 'addons.groupLlm', types: ['llm'] },
  { id: 'voice', labelKey: 'addons.groupVoice', types: ['asr', 'stt', 'voice'] },
  { id: 'dns', labelKey: 'addons.groupDns', types: ['ddns', 'dns'] },
  { id: 'mcp', labelKey: 'addons.groupMcp', types: ['mcp'] },
];

function groupForType(type: string): string {
  return ADDON_GROUPS.find((g) => g.types.includes(type))?.id ?? 'other';
}

function renderConfigField(field: AddonConfigField, value: string) {
  switch (field.fieldType) {
    case 'boolean':
      return (
        <FormField key={field.key} label={field.label} description={field.description}>
          <ToggleField
            enabled={value === 'true'}
            onChange={() => {}}
            label={field.label}
            disabled
          />
        </FormField>
      );

    case 'password':
      return (
        <FormField key={field.key} label={field.label} description={field.description}>
          <input
            type="password"
            value={value}
            readOnly
            className={inputClassName}
            placeholder={field.defaultValue ?? ''}
            disabled
          />
        </FormField>
      );

    case 'number':
      return (
        <FormField key={field.key} label={field.label} description={field.description}>
          <input type="number" value={value} readOnly className={inputClassName} disabled />
        </FormField>
      );

    case 'integer':
      return (
        <FormField key={field.key} label={field.label} description={field.description}>
          <input type="number" step={1} value={value} readOnly className={inputClassName} disabled />
        </FormField>
      );

    case 'textarea':
      return (
        <FormField key={field.key} label={field.label} description={field.description}>
          <textarea
            value={value}
            readOnly
            className={textareaClassName}
            placeholder={field.defaultValue ?? ''}
            disabled
          />
        </FormField>
      );

    case 'text':
    default:
      return (
        <FormField key={field.key} label={field.label} description={field.description}>
          <input
            type="text"
            value={value}
            readOnly
            className={inputClassName}
            placeholder={field.defaultValue ?? ''}
            disabled
          />
        </FormField>
      );
  }
}

export function AddonSection() {
  const { addToast } = useToast();
  const { t } = useTranslation();

  const [addons, setAddons] = useState<AddonInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  // Groups are collapsed by default; the user expands one by clicking its
  // header. Pre-seeded with every known group id (plus the "other"
  // fallback) so nothing starts open.
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(
    () => new Set([...ADDON_GROUPS.map((g) => g.id), 'other']),
  );
  const [editing, setEditing] = useState<AddonInfo | null>(null);
  const [draftSource, setDraftSource] = useState<string>('');
  const [sourceBusy, setSourceBusy] = useState(false);
  const [uploading, setUploading] = useState(false);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  const loadAddons = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      const result = await getAddons();
      setAddons(result);
    } catch (err) {
      setLoadError(err instanceof Error ? err.message : t('addons.loadError'));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    void loadAddons();
  }, [loadAddons]);

  const handleToggle = useCallback(
    async (addon: AddonInfo) => {
      const newEnabled = !addon.enabled;
      try {
        const updated = await updateAddonConfig(addon.id, {
          enabled: newEnabled,
          userConfig: addon.userConfig,
        });
        setAddons((prev) => prev.map((a) => (a.id === addon.id ? updated : a)));
        addToast('success', t(newEnabled ? 'addons.enabled' : 'addons.disabled', { name: addon.name }));
      } catch {
        addToast('error', t('addons.saveFailed'));
      }
    },
    [addToast, t],
  );

  const handleExpand = useCallback((addon: AddonInfo) => {
    setExpandedId((prev) => (prev === addon.id ? null : addon.id));
  }, []);

  const toggleGroup = useCallback((groupId: string) => {
    setCollapsedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(groupId)) {
        next.delete(groupId);
      } else {
        next.add(groupId);
      }
      return next;
    });
  }, []);

  const openEditor = useCallback(async (addon: AddonInfo) => {
    setEditing(addon);
    setDraftSource('');
    setSourceBusy(true);
    try {
      const detail = await getAddonDetail(addon.id);
      setDraftSource(detail.source);
    } catch {
      // If source can't load, keep the editor open with empty content so the
      // user at least sees the failure when they try to save.
    } finally {
      setSourceBusy(false);
    }
  }, []);

  const handleSaveSource = useCallback(async () => {
    if (!editing) return;
    setSourceBusy(true);
    try {
      const detail = await saveAddonSource(editing.id, draftSource);
      setAddons((prev) => prev.map((a) => (a.id === detail.id ? detail : a)));
      addToast('success', t('addons.sourceSaved', { name: editing.name }));
      setEditing(null);
    } catch (err) {
      addToast(
        'error',
        err instanceof Error
          ? `${t('addons.sourceSaveFailed')}: ${err.message}`
          : t('addons.sourceSaveFailed'),
      );
    } finally {
      setSourceBusy(false);
    }
  }, [editing, draftSource, addToast, t]);

  const handleResetSource = useCallback(async () => {
    if (!editing) return;
    setSourceBusy(true);
    try {
      const detail = await resetAddonSource(editing.id);
      setDraftSource(detail.source);
      setAddons((prev) => prev.map((a) => (a.id === detail.id ? detail : a)));
      addToast('success', t('addons.sourceReset', { name: editing.name }));
    } catch (err) {
      addToast(
        'error',
        err instanceof Error
          ? `${t('addons.sourceSaveFailed')}: ${err.message}`
          : t('addons.sourceSaveFailed'),
      );
    } finally {
      setSourceBusy(false);
    }
  }, [editing, addToast, t]);

  const handleDelete = useCallback(async () => {
    if (!editing) return;
    setSourceBusy(true);
    try {
      await deleteAddon(editing.id);
      setAddons((prev) => prev.filter((a) => a.id !== editing.id));
      addToast('success', t('addons.deleteSuccess', { name: editing.name }));
      setEditing(null);
    } catch (err) {
      addToast(
        'error',
        err instanceof Error
          ? `${t('addons.deleteFailed')}: ${err.message}`
          : t('addons.deleteFailed'),
      );
    } finally {
      setSourceBusy(false);
    }
  }, [editing, addToast, t]);

  const handleUpload = useCallback(
    async (file: File) => {
      setUploading(true);
      try {
        const installed = await uploadAddonZip(file);
        const result = await getAddons();
        setAddons(result);
        addToast('success', t('addons.uploadSuccess', { name: installed.name }));
      } catch (err) {
        addToast(
          'error',
          err instanceof Error
            ? `${t('addons.uploadFailed')}: ${err.message}`
            : t('addons.uploadFailed'),
        );
      } finally {
        setUploading(false);
      }
    },
    [addToast, t],
  );

  // Bucket addons into groups (plus an "other" fallback). Defined groups are
  // always rendered (even when empty) so the layout stays stable; "other"
  // only appears when an addon lands outside the known categories.
  const grouped = useMemo(() => {
    const buckets: Record<string, AddonInfo[]> = {};
    for (const group of ADDON_GROUPS) {
      buckets[group.id] = [];
    }
    buckets.other = [];
    for (const addon of addons) {
      const gid = groupForType(addon.addonType);
      buckets[gid].push(addon);
    }
    return buckets;
  }, [addons]);

  const visibleGroups = useMemo(() => {
    const list = ADDON_GROUPS.map((g) => ({
      id: g.id,
      labelKey: g.labelKey,
      items: grouped[g.id] ?? [],
    }));
    if ((grouped.other ?? []).length > 0) {
      list.push({ id: 'other', labelKey: 'addons.groupOther', items: grouped.other });
    }
    return list;
  }, [grouped]);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-6 h-6 animate-spin text-warm-black" />
      </div>
    );
  }

  if (loadError) {
    return (
      <div className="flex items-center gap-2 text-warm-black py-8">
        <AlertCircle className="w-5 h-5" />
        <span className="font-light">{loadError}</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SectionHeader title={t('addons.title')} description={t('addons.description')} />

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <div className="flex items-center justify-between gap-3 flex-wrap">
          <h3 className="text-sm font-semibold text-warm-black tracking-wide">
            {t('addons.installed', { count: addons.length })}
          </h3>
          <input
            ref={fileInputRef}
            type="file"
            accept=".zip,application/zip"
            className="hidden"
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (file) {
                void handleUpload(file);
              }
              // Reset so selecting the same file again re-triggers change.
              e.target.value = '';
            }}
          />
          <button
            type="button"
            onClick={() => fileInputRef.current?.click()}
            disabled={uploading}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-standard text-xs font-medium bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50"
          >
            {uploading ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : (
              <Upload className="w-3.5 h-3.5" />
            )}
            {uploading ? t('addons.uploading') : t('addons.upload')}
          </button>
        </div>

        {addons.length === 0 ? (
          <div className="rounded-comfortable border border-dashed border-golden-sand p-4 text-xs text-deep-olive">
            {t('addons.noAddons')}
          </div>
        ) : (
          <div className="space-y-5">
            {visibleGroups.map((group) => (
              <AddonGroupBlock
                key={group.id}
                groupId={group.id}
                label={t(group.labelKey)}
                addons={group.items}
                collapsed={collapsedGroups.has(group.id)}
                expandedId={expandedId}
                onToggleGroup={toggleGroup}
                onExpand={handleExpand}
                onToggle={handleToggle}
                onEdit={openEditor}
              />
            ))}
          </div>
        )}
      </div>

      {editing && (
        <SourceEditorModal
          addon={editing}
          draft={draftSource}
          busy={sourceBusy}
          onDraftChange={setDraftSource}
          onClose={() => setEditing(null)}
          onSave={handleSaveSource}
          onReset={handleResetSource}
          onDelete={handleDelete}
        />
      )}
    </div>
  );
}

interface AddonGroupBlockProps {
  groupId: string;
  label: string;
  addons: AddonInfo[];
  collapsed: boolean;
  expandedId: string | null;
  onToggleGroup: (id: string) => void;
  onExpand: (addon: AddonInfo) => void;
  onToggle: (addon: AddonInfo) => void;
  onEdit: (addon: AddonInfo) => void;
}

function AddonGroupBlock({
  groupId,
  label,
  addons,
  collapsed,
  expandedId,
  onToggleGroup,
  onExpand,
  onToggle,
  onEdit,
}: AddonGroupBlockProps) {
  const { t } = useTranslation();
  return (
    <div className="rounded-comfortable border border-golden-sand bg-pale-sand/30 overflow-hidden">
      <button
        type="button"
        onClick={() => onToggleGroup(groupId)}
        className="w-full flex items-center justify-between px-4 py-2.5 bg-pale-sand/60 hover:bg-pale-sand transition-colors duration-vt-base"
      >
        <div className="flex items-center gap-2">
          {collapsed ? (
            <ChevronDown className="w-4 h-4 text-deep-olive" />
          ) : (
            <ChevronUp className="w-4 h-4 text-deep-olive" />
          )}
          <span className="text-sm font-semibold text-warm-black">{label}</span>
        </div>
        <span className="text-[11px] font-medium text-deep-olive">
          {t('addons.groupCount', { count: addons.length })}
        </span>
      </button>

      {!collapsed && (
        <div className="p-3 space-y-3">
          {addons.length === 0 ? (
            <div className="text-[11px] text-warm-khaki px-1 py-2">
              {t('addons.noAddonsInGroup')}
            </div>
          ) : (
            addons.map((addon) => (
              <AddonCard
                key={addon.id}
                addon={addon}
                expanded={expandedId === addon.id}
                onExpand={onExpand}
                onToggle={onToggle}
                onEdit={onEdit}
              />
            ))
          )}
        </div>
      )}
    </div>
  );
}

interface AddonCardProps {
  addon: AddonInfo;
  expanded: boolean;
  onExpand: (addon: AddonInfo) => void;
  onToggle: (addon: AddonInfo) => void;
  onEdit: (addon: AddonInfo) => void;
}

function AddonCard({ addon, expanded, onExpand, onToggle, onEdit }: AddonCardProps) {
  const { t } = useTranslation();
  const { addToast } = useToast();
  const [exporting, setExporting] = useState(false);

  // Download the addon as a zip (manifest + source) — the same archive the
  // upload endpoint accepts, so it can be tweaked offline and re-uploaded.
  const handleExport = useCallback(async () => {
    if (exporting) return;
    setExporting(true);
    try {
      const blob = await exportAddonZip(addon.id);
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement('a');
      anchor.href = url;
      anchor.download = `${addon.id}.zip`;
      document.body.appendChild(anchor);
      anchor.click();
      anchor.remove();
      URL.revokeObjectURL(url);
      addToast('success', t('addons.exportSuccess', { name: addon.name }));
    } catch (err) {
      addToast('error', err instanceof Error ? err.message : t('addons.exportFailed'));
    } finally {
      setExporting(false);
    }
  }, [addToast, addon.id, addon.name, exporting, t]);

  return (
    <div className="rounded-comfortable border border-golden-sand bg-pale-sand/40">
      {/* Header row */}
      <div className="flex items-center justify-between p-4 gap-2">
        <div className="flex items-center gap-3 flex-1 min-w-0">
          <button
            type="button"
            onClick={() => onExpand(addon)}
            className="p-1 rounded-standard text-deep-olive hover:text-warm-black hover:bg-golden-cream transition-colors duration-vt-base shrink-0"
            aria-label={expanded ? 'Collapse' : 'Expand'}
          >
            {expanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
          </button>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <h4 className="text-sm font-semibold text-warm-black truncate">{addon.name}</h4>
              <span className="inline-flex items-center px-2 py-0.5 rounded-standard text-[11px] font-medium bg-accent-subtle text-accent-text border border-accent-subtle">
                {addon.addonType}
              </span>
              {addon.builtin ? (
                <span className="inline-flex items-center px-2 py-0.5 rounded-standard text-[11px] font-medium bg-warm-black/10 text-deep-olive border border-warm-black/10">
                  {t('addons.builtin')}
                </span>
              ) : (
                <span className="inline-flex items-center px-2 py-0.5 rounded-standard text-[11px] font-medium bg-amber-gold/20 text-warm-black border border-amber-gold/40">
                  {t('addons.uploaded')}
                </span>
              )}
              {addon.modified && (
                <span className="inline-flex items-center px-2 py-0.5 rounded-standard text-[11px] font-medium bg-vt-warning/15 text-warm-black border border-vt-warning/60">
                  {t('addons.modified')}
                </span>
              )}
            </div>
            <p className="text-xs text-deep-olive mt-0.5 truncate">{addon.description}</p>
          </div>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <button
            type="button"
            onClick={() => void handleExport()}
            disabled={exporting}
            className="inline-flex items-center gap-1 px-2 py-1 rounded-standard text-[11px] font-medium text-deep-olive hover:text-warm-black hover:bg-golden-cream border border-transparent hover:border-golden-sand transition-colors duration-vt-base disabled:opacity-50"
            title={t('addons.export')}
          >
            {exporting ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : (
              <Download className="w-3.5 h-3.5" />
            )}
            <span className="hidden sm:inline">{t('addons.export')}</span>
          </button>
          <button
            type="button"
            onClick={() => onEdit(addon)}
            className="inline-flex items-center gap-1 px-2 py-1 rounded-standard text-[11px] font-medium text-deep-olive hover:text-warm-black hover:bg-golden-cream border border-transparent hover:border-golden-sand transition-colors duration-vt-base"
            title={t('addons.editSource')}
          >
            <Pencil className="w-3.5 h-3.5" />
            <span className="hidden sm:inline">{t('addons.editSource')}</span>
          </button>
          <ToggleField enabled={addon.enabled} onChange={() => onToggle(addon)} />
        </div>
      </div>

      {/* Expanded content */}
      {expanded && (
        <div className="px-4 pb-4 pt-0 space-y-5 border-t border-golden-sand/60 mt-0 pt-4">
          {/* Description */}
          <div>
            <h4 className="text-xs font-semibold text-warm-black mb-1">
              {t('addons.detailDescription')}
            </h4>
            <p className="text-xs text-deep-olive">{addon.description}</p>
            <p className="text-[11px] text-warm-khaki mt-1">
              {t('addons.author')}: {addon.author}
              {addon.email.trim() !== '' && (
                <a
                  href={`mailto:${addon.email.trim()}`}
                  className="ml-1 underline decoration-warm-khaki/60 hover:text-warm-black"
                  onClick={(e) => e.stopPropagation()}
                >
                  {'<'}
                  {addon.email.trim()}
                  {'>'}
                </a>
              )}
              {addon.date.trim() !== '' && <span className="ml-1">· {addon.date.trim()}</span>}
            </p>
          </div>

          {/* Config fields (readonly) */}
          {addon.configFields.length > 0 && (
            <div className="space-y-3">
              <h4 className="text-xs font-semibold text-warm-black">{t('addons.configuration')}</h4>
              {addon.configFields.map((field) =>
                renderConfigField(field, addon.userConfig[field.key] ?? field.defaultValue ?? ''),
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

interface SourceEditorModalProps {
  addon: AddonInfo;
  draft: string;
  busy: boolean;
  onDraftChange: (value: string) => void;
  onClose: () => void;
  onSave: () => void;
  onReset: () => void;
  onDelete: () => void;
}

function SourceEditorModal({
  addon,
  draft,
  busy,
  onDraftChange,
  onClose,
  onSave,
  onReset,
  onDelete,
}: SourceEditorModalProps) {
  const { t } = useTranslation();
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      <div
        className="fixed inset-0 bg-warm-black/50"
        onClick={busy ? undefined : onClose}
        style={{ backdropFilter: 'blur(4px)' }}
      />
      <div className="relative w-full max-w-3xl h-[80vh] flex flex-col rounded-comfortable bg-warm-ivory border border-golden-sand shadow-lg">
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-3 border-b border-golden-sand">
          <div className="min-w-0">
            <h3 className="text-sm font-semibold text-warm-black truncate">
              {t('addons.editSourceTitle', { name: addon.name })}
            </h3>
            <p className="text-[11px] text-deep-olive mt-0.5">{t('addons.editSourceHint')}</p>
          </div>
          <button
            type="button"
            onClick={onClose}
            disabled={busy}
            className="p-1 rounded-standard text-deep-olive hover:text-warm-black hover:bg-golden-cream transition-colors duration-vt-base disabled:opacity-50"
            aria-label={t('common.cancel')}
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Editor body */}
        <div className="flex-1 min-h-0 p-3">
          {busy && draft === '' ? (
            <div className="flex items-center justify-center h-full">
              <Loader2 className="w-5 h-5 animate-spin text-warm-black" />
            </div>
          ) : (
            <textarea
              value={draft}
              onChange={(e) => onDraftChange(e.target.value)}
              spellCheck={false}
              disabled={busy}
              className="w-full h-full bg-warm-black/5 border border-golden-sand rounded-standard p-3 text-[12px] font-mono text-warm-black focus:outline-none focus:border-amber-gold resize-none"
            />
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between gap-2 px-5 py-3 border-t border-golden-sand">
          <div>
            {!addon.builtin ? (
              <button
                type="button"
                onClick={onDelete}
                disabled={busy}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-standard text-xs font-medium bg-vt-error text-text-inverse border border-vt-error hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50"
              >
                <Trash2 className="w-3.5 h-3.5" />
                {t('addons.delete')}
              </button>
            ) : (
              <button
                type="button"
                onClick={onReset}
                disabled={busy || !addon.modified}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-standard text-xs font-medium bg-golden-cream text-warm-black border border-golden-sand hover:bg-pale-sand transition-colors duration-vt-base disabled:opacity-50"
                title={t('addons.resetSource')}
              >
                <RotateCcw className="w-3.5 h-3.5" />
                {t('addons.resetSource')}
              </button>
            )}
          </div>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={onClose}
              disabled={busy}
              className="inline-flex items-center justify-center px-4 py-1.5 rounded-standard text-xs font-medium bg-golden-cream text-warm-black border border-golden-sand hover:bg-pale-sand transition-colors duration-vt-base disabled:opacity-50"
            >
              {t('common.cancel')}
            </button>
            <button
              type="button"
              onClick={onSave}
              disabled={busy}
              className="inline-flex items-center justify-center gap-1.5 px-4 py-1.5 rounded-standard text-xs font-medium bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50"
            >
              {busy ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Save className="w-3.5 h-3.5" />}
              {t('addons.saveSource')}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
