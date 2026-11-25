import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Loader2,
  AlertCircle,
  AlertTriangle,
  ChevronDown,
  ChevronUp,
  Plus,
  RotateCcw,
  Save,
  Trash2,
} from 'lucide-react';
import {
  getMcpConfig,
  updateMcpConfig,
  getMcpEndpoints,
  getMcpTemplates,
  saveMcpTemplate,
  deleteMcpTemplate,
} from '@/api/mcp';
import { useConfigSection } from '@/hooks/use-config-section';
import { useToast } from '@/components/shared/Toast';
import { CopyButton } from '@/components/shared/CopyButton';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { FormField } from '@/components/shared/FormField';
import { ToggleField } from '@/components/shared/ToggleField';
import { NumberInput } from '@/components/shared/NumberInput';
import { SelectField } from '@/components/shared/SelectField';
import { SecretField } from '@/components/shared/SecretField';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { SaveResetButtons } from '@/components/shared/SaveResetButtons';
import { useTranslation } from '@/i18n';
import { generateRandomToken } from '@/lib/random-token';
import { renderMcpTemplate } from '@/lib/mcp-template';
import type { McpConfig, McpEndpoint, McpTemplate, McpTemplateUpdate } from '@/types';

// Friendly names for well-known endpoint ids (`/{id}/mcp` paths).
const ENDPOINT_NAMES: Record<string, string> = {
  terminal: 'Terminal MCP',
  easyeda: 'EasyEDA Pro MCP',
};

export function McpSection() {
  const { addToast } = useToast();
  const { t } = useTranslation();

  const fetchFn = useCallback(() => getMcpConfig(), []);
  const saveFn = useCallback((data: McpConfig) => updateMcpConfig(data), []);

  const { data, loading, saving, loadError, saveError, isDirty, updateField, save, reset } =
    useConfigSection<McpConfig>(fetchFn, saveFn, { autoSave: false });

  // The access token is mandatory (the server generates a random one by
  // default and refuses to store an empty value), so a blank or
  // whitespace-only field blocks saving with an inline error.
  const tokenInvalid = (data?.accessToken ?? '').trim().length === 0;

  // The token must never be empty by default: the store repairs legacy
  // configs on the server side, but if an empty one still reaches the form
  // (e.g. an older server build), pre-fill a generated token so the field is
  // usable immediately and saving stays unblocked. The user reviews it
  // before saving, same as pressing the dice button manually.
  const accessToken = data?.accessToken;
  useEffect(() => {
    if ((accessToken ?? '').trim().length === 0) {
      updateField('accessToken', generateRandomToken());
    }
  }, [accessToken, updateField]);

  const handleSave = async () => {
    if (tokenInvalid) return;
    const ok = await save();
    if (ok) {
      addToast('success', t('mcp.saved'));
    }
  };

  // Compute the terminal MCP endpoint URL from the current (possibly unsaved)
  // form values. The host defaults to localhost for display regardless of bind
  // address; the path `/terminal/mcp` mirrors `TerminalService::path()`.
  const baseUrl = useMemo(() => {
    const port = data?.port ?? 6180;
    return `http://localhost:${port}`;
  }, [data?.port]);

  // Friendly display names for the endpoints this UI knows about; anything
  // else falls back to the server-provided name.
  const endpointDisplayName = (endpoint: McpEndpoint): string =>
    ENDPOINT_NAMES[endpoint.id] ?? endpoint.name;

  // Every endpoint the MCP host currently serves (builtin services + live
  // type:"mcp" addons), reported by the server so the list matches what is
  // actually mounted. The terminal endpoint is always listed first even if
  // the server report lags. Polled so the addon daemon badge tracks bridge
  // connections/disconnections live.
  const [activeEndpoints, setActiveEndpoints] = useState<McpEndpoint[]>([]);
  useEffect(() => {
    if (!data?.enabled) return;
    let cancelled = false;
    const refresh = () => {
      getMcpEndpoints()
        .then((result) => {
          if (!cancelled) {
            const others = result.filter(
              (endpoint) => endpoint.id !== 'terminal',
            );
            const terminal = result.find((endpoint) => endpoint.id === 'terminal');
            setActiveEndpoints([
              terminal ?? { id: 'terminal', name: 'Terminal MCP', kind: 'builtin' },
              ...others,
            ]);
          }
        })
        .catch(() => {
          // The list is informational; keep the previous one on failure.
        });
    };
    refresh();
    const interval = setInterval(refresh, 5000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [data?.enabled]);

  // App list for the endpoint copy control: picking an app copies its rendered
  // setup line (a shell command or chat prompt) to the clipboard. It is
  // rendered client-side from the current form values so it tracks unsaved
  // edits to the port and access token immediately, and parameterized by
  // endpoint URL so every row can render its own setup line.
  const [appTemplates, setAppTemplates] = useState<McpTemplate[]>([]);
  const [selectedAppId, setSelectedAppId] = useState('');
  const loadAppTemplates = useCallback(async () => {
    try {
      const result = await getMcpTemplates();
      setAppTemplates(result);
      setSelectedAppId((prev) =>
        result.some((template) => template.id === prev) ? prev : (result[0]?.id ?? ''),
      );
    } catch {
      // Keep the previous list; the templates card surfaces load errors.
    }
  }, []);
  useEffect(() => {
    if (data?.enabled) void loadAppTemplates();
  }, [data?.enabled, loadAppTemplates]);
  const renderForUrl = useCallback(
    (url: string) => {
      const template = appTemplates.find((t) => t.id === selectedAppId);
      if (!template) return '';
      return renderMcpTemplate(
        template.template,
        template.executable,
        url,
        data?.accessToken ?? '',
      );
    },
    [appTemplates, selectedAppId, data?.accessToken],
  );
  const endpointUrl = `${baseUrl}/terminal/mcp`;
  // Setup notes for apps whose flow needs more than the copied line itself
  // (chat-prompt setup like Kimi, Pi, and OpenCode, or environment prerequisites).
  const selectedDescription =
    appTemplates.find((t) => t.id === selectedAppId)?.description?.trim() ?? '';

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-6 h-6 animate-spin text-warm-black" />
      </div>
    );
  }

  if (loadError || !data) {
    return (
      <div className="flex items-center gap-2 text-warm-black py-8">
        <AlertCircle className="w-5 h-5" />
        <span className="font-light">{loadError ?? t('mcp.loadError')}</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SectionHeader title={t('mcp.title')} description={t('mcp.description')} />

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <ToggleField
          enabled={data.enabled}
          onChange={(v) => updateField('enabled', v)}
          label={t('mcp.enable')}
        />

        {data.enabled && (
          <>
            <FormField label={t('mcp.port')}>
              <NumberInput
                value={data.port}
                onChange={(v) => updateField('port', v)}
                min={1}
                max={65535}
              />
            </FormField>

            <FormField label={t('mcp.bindAddress')}>
              <SelectField
                value={data.bindAddress}
                onChange={(e) => updateField('bindAddress', e.target.value)}
                options={[
                  { value: '127.0.0.1', label: t('mcp.bindLocalhost') },
                  { value: '0.0.0.0', label: t('mcp.bindAll') },
                ]}
              />
            </FormField>

            <FormField
              label={t('mcp.accessToken')}
              description={t('mcp.accessTokenHint')}
              error={tokenInvalid ? t('mcp.tokenRequired') : undefined}
            >
              <SecretField
                value={data.accessToken}
                onChange={(e) => updateField('accessToken', e.target.value)}
                // Copy without revealing: the server echoes the stored token
                // back, so it can go straight to the clipboard while the
                // field stays masked.
                copyText={data.accessToken}
                copyTitle={t('mcp.copyToken')}
                onGenerate={() => updateField('accessToken', generateRandomToken())}
                generateTitle={t('mcp.generateToken')}
              />
            </FormField>

            <FormField
              label={t('mcp.streamThrottle')}
              description={t('mcp.streamThrottleHint')}
            >
              <NumberInput
                value={data.streamThrottleMs}
                onChange={(v) => updateField('streamThrottleMs', v)}
                min={0}
              />
            </FormField>

            <FormField label={t('mcp.endpoint')} description={t('mcp.endpointHint')}>
              <div className="space-y-2">
                <div className="flex items-center gap-2">
                  <SelectField
                    value={selectedAppId}
                    onChange={(e) => setSelectedAppId(e.target.value)}
                    options={appTemplates.map((template) => ({
                      value: template.id,
                      label: template.displayName,
                    }))}
                    ariaLabel={t('mcp.selectApp')}
                    className="w-40 shrink-0"
                  />
                </div>
                {activeEndpoints.map((endpoint) => {
                  const url = `${baseUrl}/${endpoint.id}/mcp`;
                  return (
                    <div key={endpoint.id} className="flex items-center gap-2">
                      <span
                        className="w-32 shrink-0 text-xs text-deep-olive truncate"
                        title={endpointDisplayName(endpoint)}
                      >
                        {endpointDisplayName(endpoint)}
                      </span>
                      <code className="flex-1 px-3 py-2 rounded-standard bg-warm-black/5 text-sm text-warm-black font-mono break-all">
                        {url}
                      </code>
                      {endpoint.daemonConnected !== undefined && (
                        <span
                          className={`shrink-0 inline-flex items-center px-2 py-0.5 rounded-standard text-[11px] font-medium border ${
                            endpoint.daemonConnected
                              ? 'bg-vt-success/25 text-warm-black border-vt-success'
                              : 'bg-vt-error/15 text-warm-black border-vt-error/60'
                          }`}
                        >
                          {endpoint.daemonConnected
                            ? t('mcp.daemonConnected')
                            : t('mcp.daemonDisconnected')}
                        </span>
                      )}
                      <CopyButton
                        text={renderForUrl(url)}
                        title={t('mcp.copyConfig')}
                        className="p-2 rounded-standard text-deep-olive hover:text-warm-black hover:bg-golden-sand/60 transition-colors duration-vt-base"
                      />
                    </div>
                  );
                })}
              </div>
              {selectedDescription && (
                <p className="mt-2 text-xs text-deep-olive leading-relaxed">
                  {selectedDescription}
                </p>
              )}
            </FormField>
          </>
        )}

        {/* Save/reset belong to this card's settings, so they live inside it
          rather than at the page bottom (below the templates card). */}
        <SaveResetButtons
          onSave={handleSave}
          onReset={reset}
          saving={saving}
          isDirty={isDirty}
          saveDisabled={tokenInvalid}
          className="pt-0"
        />

        {saveError && <div className="text-xs text-vt-error">{saveError}</div>}
      </div>

      {data.enabled && (
        <McpTemplatesCard
          previewUrl={endpointUrl}
          previewToken={data.accessToken}
          onTemplatesChanged={() => void loadAppTemplates()}
        />
      )}
    </div>
  );
}

const inputClassName =
  'w-full px-2 py-1.5 bg-bg-inset border border-golden-sand rounded-standard text-warm-black text-sm placeholder:text-warm-khaki focus:outline-none focus:border-amber-gold disabled:opacity-50';

const textareaClassName =
  'w-full px-2 py-1.5 bg-bg-inset border border-golden-sand rounded-standard text-warm-black text-sm font-mono placeholder:text-warm-khaki focus:outline-none focus:border-amber-gold disabled:opacity-50 resize-y min-h-[80px]';

interface TemplateDraft {
  displayName: string;
  executable: string;
  template: string;
}

function draftFromTemplate(template: McpTemplate): TemplateDraft {
  return {
    displayName: template.displayName,
    executable: template.executable,
    template: template.template,
  };
}

function draftToUpdate(draft: TemplateDraft): McpTemplateUpdate {
  return {
    displayName: draft.displayName,
    executable: draft.executable,
    template: draft.template,
  };
}

function errorText(t: (key: string) => string, key: string, err: unknown): string {
  return err instanceof Error ? `${t(key)}: ${err.message}` : t(key);
}

interface McpTemplatesCardProps {
  /** Rendered-command inputs reflecting the current (possibly unsaved) form
   * values, shared with the endpoint copy control so every preview in the
   * section stays in sync. */
  previewUrl: string;
  previewToken: string;
  /** Invoked after the template list changes (save/delete/reset). */
  onTemplatesChanged?: () => void;
}

function McpTemplatesCard({ previewUrl, previewToken, onTemplatesChanged }: McpTemplatesCardProps) {
  const { addToast } = useToast();
  const { t } = useTranslation();

  const [templates, setTemplates] = useState<McpTemplate[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [addingNew, setAddingNew] = useState(false);
  const [pendingDelete, setPendingDelete] = useState<McpTemplate | null>(null);
  const [pendingReset, setPendingReset] = useState<McpTemplate | null>(null);
  const [confirmBusy, setConfirmBusy] = useState(false);

  const loadTemplates = useCallback(async () => {
    setLoadError(null);
    try {
      const result = await getMcpTemplates();
      setTemplates(result);
    } catch (err) {
      setLoadError(err instanceof Error ? err.message : t('mcp.templatesLoadError'));
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    void loadTemplates();
  }, [loadTemplates]);

  const handleChanged = useCallback(() => {
    void loadTemplates();
    onTemplatesChanged?.();
  }, [loadTemplates, onTemplatesChanged]);

  const handleDelete = useCallback(async () => {
    if (!pendingDelete || confirmBusy) return;
    setConfirmBusy(true);
    try {
      await deleteMcpTemplate(pendingDelete.id);
      addToast('success', t('mcp.templatesDeleted'));
      setPendingDelete(null);
      setExpandedId((prev) => (prev === pendingDelete.id ? null : prev));
      handleChanged();
    } catch (err) {
      addToast('error', errorText(t, 'mcp.templatesDeleteFailed', err));
    } finally {
      setConfirmBusy(false);
    }
  }, [pendingDelete, confirmBusy, addToast, t, handleChanged]);

  const handleReset = useCallback(async () => {
    if (!pendingReset || confirmBusy) return;
    setConfirmBusy(true);
    try {
      await deleteMcpTemplate(pendingReset.id);
      addToast('success', t('mcp.templatesResetDone'));
      setPendingReset(null);
      handleChanged();
    } catch (err) {
      addToast('error', errorText(t, 'mcp.templatesResetFailed', err));
    } finally {
      setConfirmBusy(false);
    }
  }, [pendingReset, confirmBusy, addToast, t, handleChanged]);

  return (
    <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
      <div className="flex items-center justify-between gap-3 flex-wrap">
        <div className="min-w-0">
          <h3 className="text-sm font-semibold text-warm-black tracking-wide">
            {t('mcp.templatesTitle')}
          </h3>
          <p className="text-xs text-deep-olive mt-0.5">{t('mcp.templatesDescription')}</p>
        </div>
        <button
          type="button"
          onClick={() => {
            setAddingNew(true);
            setExpandedId(null);
          }}
          disabled={addingNew}
          className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-standard text-xs font-medium bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50"
        >
          <Plus className="w-3.5 h-3.5" />
          {t('mcp.templatesAddApp')}
        </button>
      </div>

      {loading ? (
        <div className="flex items-center justify-center py-6">
          <Loader2 className="w-5 h-5 animate-spin text-warm-black" />
        </div>
      ) : loadError ? (
        <div className="flex items-center gap-2 text-warm-black py-2">
          <AlertCircle className="w-4 h-4" />
          <span className="text-xs font-light">{loadError}</span>
        </div>
      ) : (
        <div className="space-y-3">
          {addingNew && (
            <NewTemplateRow
              onSaved={() => {
                setAddingNew(false);
                handleChanged();
              }}
              onCancel={() => setAddingNew(false)}
            />
          )}
          {templates.map((template) => (
            <TemplateRow
              // `modified` in the key re-initializes the draft after a reset.
              key={`${template.id}:${template.modified ? 1 : 0}`}
              template={template}
              previewUrl={previewUrl}
              previewToken={previewToken}
              expanded={expandedId === template.id}
              onToggleExpand={() =>
                setExpandedId((prev) => (prev === template.id ? null : template.id))
              }
              onSaved={handleChanged}
              onRequestDelete={() => setPendingDelete(template)}
              onRequestReset={() => setPendingReset(template)}
            />
          ))}
        </div>
      )}

      <ConfirmDialog
        open={pendingDelete !== null}
        variant="danger"
        title={t('mcp.templatesDeleteConfirmTitle')}
        message={t('mcp.templatesDeleteConfirmMessage', {
          name: pendingDelete?.displayName ?? '',
        })}
        onConfirm={() => void handleDelete()}
        onCancel={() => setPendingDelete(null)}
      />
      <ConfirmDialog
        open={pendingReset !== null}
        title={t('mcp.templatesResetConfirmTitle')}
        message={t('mcp.templatesResetConfirmMessage', {
          name: pendingReset?.displayName ?? '',
        })}
        onConfirm={() => void handleReset()}
        onCancel={() => setPendingReset(null)}
      />
    </div>
  );
}

interface TemplateFieldsProps {
  draft: TemplateDraft;
  onDraftChange: (draft: TemplateDraft) => void;
  disabled?: boolean;
}

function TemplateFields({ draft, onDraftChange, disabled }: TemplateFieldsProps) {
  const { t } = useTranslation();
  const update = (patch: Partial<TemplateDraft>) => onDraftChange({ ...draft, ...patch });

  return (
    <>
      <FormField label={t('mcp.templatesDisplayName')}>
        <input
          type="text"
          value={draft.displayName}
          onChange={(e) => update({ displayName: e.target.value })}
          disabled={disabled}
          className={inputClassName}
        />
      </FormField>

      <FormField label={t('mcp.templatesExecutable')}>
        <input
          type="text"
          value={draft.executable}
          onChange={(e) => update({ executable: e.target.value })}
          disabled={disabled}
          className={inputClassName}
        />
      </FormField>

      <FormField label={t('mcp.templatesTemplate')} description={t('mcp.templatesTemplateHint')}>
        <textarea
          value={draft.template}
          onChange={(e) => update({ template: e.target.value })}
          spellCheck={false}
          disabled={disabled}
          className={textareaClassName}
        />
      </FormField>
    </>
  );
}

interface TemplateRowProps {
  template: McpTemplate;
  previewUrl: string;
  previewToken: string;
  expanded: boolean;
  onToggleExpand: () => void;
  onSaved: () => void;
  onRequestDelete: () => void;
  onRequestReset: () => void;
}

function TemplateRow({
  template,
  previewUrl,
  previewToken,
  expanded,
  onToggleExpand,
  onSaved,
  onRequestDelete,
  onRequestReset,
}: TemplateRowProps) {
  const { addToast } = useToast();
  const { t } = useTranslation();
  const [draft, setDraft] = useState<TemplateDraft>(() => draftFromTemplate(template));
  const [saving, setSaving] = useState(false);

  // The preview follows the draft while editing, and the shared form values
  // (URL/token) otherwise — both update immediately, unlike a server fetch.
  const preview = renderMcpTemplate(
    expanded ? draft.template : template.template,
    expanded ? draft.executable : template.executable,
    previewUrl,
    previewToken,
  );

  const handleSave = async () => {
    setSaving(true);
    try {
      await saveMcpTemplate(template.id, draftToUpdate(draft));
      addToast('success', t('mcp.templatesSaved'));
      onSaved();
    } catch (err) {
      addToast('error', errorText(t, 'mcp.templatesSaveFailed', err));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="rounded-comfortable border border-golden-sand bg-pale-sand/40">
      {/* Header row */}
      <div className="flex items-center justify-between p-4 gap-2">
        <div className="flex items-center gap-3 flex-1 min-w-0">
          <button
            type="button"
            onClick={onToggleExpand}
            className="p-1 rounded-standard text-deep-olive hover:text-warm-black hover:bg-golden-cream transition-colors duration-vt-base shrink-0"
            aria-label={expanded ? 'Collapse' : 'Expand'}
          >
            {expanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
          </button>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <h4 className="text-sm font-semibold text-warm-black truncate">
                {template.displayName}
              </h4>
              {template.modified && (
                <span className="inline-flex items-center px-2 py-0.5 rounded-standard text-[11px] font-medium bg-vt-warning/15 text-warm-black border border-vt-warning/60">
                  {t('mcp.templatesModified')}
                </span>
              )}
            </div>
            <p className="text-xs text-deep-olive font-mono mt-0.5 truncate">
              {template.executable}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2 shrink-0">
          {!template.builtin && (
            <button
              type="button"
              onClick={onRequestDelete}
              className="p-1.5 rounded-standard text-vt-error hover:bg-golden-cream transition-colors duration-vt-base"
              title={t('mcp.templatesDelete')}
            >
              <Trash2 className="w-3.5 h-3.5" />
            </button>
          )}
          {template.builtin && template.modified && (
            <button
              type="button"
              onClick={onRequestReset}
              className="p-1.5 rounded-standard text-deep-olive hover:text-warm-black hover:bg-golden-cream transition-colors duration-vt-base"
              title={t('mcp.templatesReset')}
            >
              <RotateCcw className="w-3.5 h-3.5" />
            </button>
          )}
        </div>
      </div>

      {/* Expanded editor */}
      {expanded && (
        <div className="px-4 pb-4 space-y-4 border-t border-golden-sand/60 pt-4">
          <TemplateFields
            draft={draft}
            onDraftChange={setDraft}
            disabled={saving}
          />

          {/* Read-only setup notes, directly under the template field and
            highlighted as a warning: chat-prompt setup and environment
            prerequisites need extra attention. Hidden when the app has none;
            notes come from the server and are not editable here. */}
          {template.description.trim() !== '' && (
            <div className="flex items-start gap-2 rounded-standard border border-vt-warning bg-vt-warning/25 px-3 py-2">
              <AlertTriangle className="w-3.5 h-3.5 shrink-0 mt-0.5 text-warm-black" />
              <p className="text-xs text-warm-black leading-relaxed">
                {template.description}
              </p>
            </div>
          )}

          <FormField label={t('mcp.templatesPreview')}>
            <div className="flex items-start gap-2">
              <code className="flex-1 px-3 py-2 rounded-standard bg-warm-black/5 text-xs text-warm-black font-mono break-all whitespace-pre-wrap">
                {preview}
              </code>
              <CopyButton
                text={preview}
                className="p-2 rounded-standard text-deep-olive hover:text-warm-black hover:bg-golden-sand/60 transition-colors duration-vt-base shrink-0"
              />
            </div>
          </FormField>

          <div className="flex justify-end">
            <button
              type="button"
              onClick={() => void handleSave()}
              disabled={saving}
              className="inline-flex items-center justify-center gap-1.5 px-4 py-1.5 rounded-standard text-xs font-medium bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50"
            >
              {saving ? (
                <Loader2 className="w-3.5 h-3.5 animate-spin" />
              ) : (
                <Save className="w-3.5 h-3.5" />
              )}
              {t('mcp.templatesSave')}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

interface NewTemplateRowProps {
  onSaved: () => void;
  onCancel: () => void;
}

function NewTemplateRow({ onSaved, onCancel }: NewTemplateRowProps) {
  const { addToast } = useToast();
  const { t } = useTranslation();
  const [id, setId] = useState('');
  const [draft, setDraft] = useState<TemplateDraft>({
    displayName: '',
    executable: '',
    template: '',
  });
  const [saving, setSaving] = useState(false);

  const idValid = /^[a-z0-9][a-z0-9-_]*$/.test(id.trim());

  const handleSave = async () => {
    if (!idValid) return;
    setSaving(true);
    try {
      await saveMcpTemplate(id.trim(), draftToUpdate(draft));
      addToast('success', t('mcp.templatesSaved'));
      onSaved();
    } catch (err) {
      addToast('error', errorText(t, 'mcp.templatesSaveFailed', err));
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="rounded-comfortable border border-amber-gold/60 bg-pale-sand/40 p-4 space-y-4">
      <FormField label={t('mcp.templatesNewAppId')} description={t('mcp.templatesNewAppIdHint')}>
        <input
          type="text"
          value={id}
          onChange={(e) => setId(e.target.value)}
          placeholder="my-agent"
          disabled={saving}
          className={inputClassName}
        />
      </FormField>

      <TemplateFields
        draft={draft}
        onDraftChange={setDraft}
        disabled={saving}
      />

      <div className="flex items-center justify-end gap-2">
        <button
          type="button"
          onClick={onCancel}
          disabled={saving}
          className="inline-flex items-center justify-center px-4 py-1.5 rounded-standard text-xs font-medium bg-golden-cream text-warm-black border border-golden-sand hover:bg-pale-sand transition-colors duration-vt-base disabled:opacity-50"
        >
          {t('common.cancel')}
        </button>
        <button
          type="button"
          onClick={() => void handleSave()}
          disabled={saving || !idValid}
          className="inline-flex items-center justify-center gap-1.5 px-4 py-1.5 rounded-standard text-xs font-medium bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50"
        >
          {saving ? (
            <Loader2 className="w-3.5 h-3.5 animate-spin" />
          ) : (
            <Save className="w-3.5 h-3.5" />
          )}
          {t('mcp.templatesSave')}
        </button>
      </div>
    </div>
  );
}
