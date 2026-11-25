import { useCallback, useState } from 'react';
import { Loader2, AlertCircle, Plus, RefreshCw, ShieldCheck } from 'lucide-react';
import {
  getPortForwardingConfig,
  updatePortForwardingConfig,
  createPortForward,
  stopPortForward,
} from '@/api/port-forwarding';
import { useConfigSection } from '@/hooks/use-config-section';
import { useToast } from '@/components/shared/Toast';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { FormField } from '@/components/shared/FormField';
import { NumberInput } from '@/components/shared/NumberInput';
import { ToggleField } from '@/components/shared/ToggleField';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { PortForwardList } from '@/components/port-forwarding/PortForwardList';
import { useTranslation } from '@/i18n';
import { cn } from '@/lib/cn';
import type { PortForwardingConfig } from '@/types';

export function PortForwardingSection() {
  const { addToast } = useToast();
  const { t } = useTranslation();
  const [stopTarget, setStopTarget] = useState<number | null>(null);
  const [creating, setCreating] = useState(false);
  const [newTargetPort, setNewTargetPort] = useState(0);
  const [createError, setCreateError] = useState<string | null>(null);

  const fetchFn = useCallback(() => getPortForwardingConfig(), []);
  const saveFn = useCallback(
    (data: PortForwardingConfig) =>
      updatePortForwardingConfig({
        enabled: data.enabled,
        idleTimeoutMinutes: data.idleTimeoutMinutes,
        portRangeStart: data.portRangeStart,
        portRangeEnd: data.portRangeEnd,
      }),
    []
  );

  const { data, loading, saving, loadError, saveError, updateField } =
    useConfigSection<PortForwardingConfig>(fetchFn, saveFn);

  const handleStop = async (proxyPort: number) => {
    try {
      await stopPortForward(proxyPort);
      addToast('success', t('portForward.stopped', { port: proxyPort }));
    } catch {
      addToast('error', t('portForward.stopFailed'));
    }
    const updated = await getPortForwardingConfig().catch(() => null);
    if (updated) {
      updateField('forwards', updated.forwards);
    }
    setStopTarget(null);
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (creating || newTargetPort < 1 || newTargetPort > 65535) return;
    setCreating(true);
    setCreateError(null);
    try {
      const forward = await createPortForward({ targetPort: newTargetPort });
      addToast(
        'success',
        t('portForward.created', { target: forward.targetPort, proxy: forward.proxyPort }),
      );
      setNewTargetPort(0);
      const updated = await getPortForwardingConfig();
      updateField('forwards', updated.forwards);
    } catch (err) {
      const message = err instanceof Error && err.message ? err.message : null;
      setCreateError(message ?? t('portForward.addFailed'));
    } finally {
      setCreating(false);
    }
  };

  const handleRefresh = async () => {
    const updated = await getPortForwardingConfig();
    updateField('forwards', updated.forwards);
    addToast('info', t('portForward.refreshed'));
  };

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
        <span className="font-light">{loadError ?? t('portForward.loadError')}</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SectionHeader title={t('portForward.title')} description={t('portForward.description')}>
        <button
          type="button"
          onClick={handleRefresh}
          className="inline-flex items-center gap-2 px-4 py-1.5 bg-golden-cream text-warm-black rounded-standard font-medium text-xs hover:bg-pale-sand transition-colors duration-vt-base border border-golden-sand"
        >
          <RefreshCw className="w-3.5 h-3.5" />
          {t('common.refresh')}
        </button>
      </SectionHeader>

      <div className="flex gap-3 rounded-comfortable border border-golden-sand bg-golden-cream p-4 md:p-5">
        <ShieldCheck className="mt-0.5 h-5 w-5 shrink-0 text-deep-olive" />
        <div className="space-y-2">
          <h3 className="text-sm font-semibold text-warm-black">
            {t('portForward.securityTitle')}
          </h3>
          <p className="text-xs font-light leading-relaxed text-warm-khaki">
            {t('portForward.securityDescription')}
          </p>
          <p className="text-xs font-light leading-relaxed text-warm-khaki">
            {t('portForward.securityUsage')}
          </p>
        </div>
      </div>

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <ToggleField
          enabled={data.enabled}
          onChange={(value) => updateField('enabled', value)}
          label={t('portForward.enable')}
        />
        <FormField
          label={t('portForward.idleTimeout')}
          description={t('portForward.idleTimeoutHint')}
        >
          <NumberInput
            value={data.idleTimeoutMinutes}
            onChange={(value) => updateField('idleTimeoutMinutes', value)}
            min={0}
            step={1}
          />
        </FormField>
      </div>

      {/* Port range config */}
      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <h3 className="text-sm font-semibold text-warm-black tracking-wide">
          {t('portForward.portRange')}
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <FormField label={t('portForward.rangeStart')}>
            <NumberInput
              value={data.portRangeStart}
              onChange={(v) => updateField('portRangeStart', v)}
              min={1024}
              max={65535}
            />
          </FormField>
          <FormField label={t('portForward.rangeEnd')}>
            <NumberInput
              value={data.portRangeEnd}
              onChange={(v) => updateField('portRangeEnd', v)}
              min={1024}
              max={65535}
            />
          </FormField>
        </div>
      </div>

      {/* Live forwards */}
      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <h3 className="text-sm font-semibold text-warm-black tracking-wide">
          {t('portForward.activeForwards', { count: data.forwards.length })}
        </h3>
        <form onSubmit={handleCreate}>
          <FormField
            label={t('portForward.targetPort')}
            description={t('portForward.targetPortHint')}
            error={createError ?? undefined}
          >
            <div className="flex flex-wrap items-center gap-3">
              <NumberInput
                value={newTargetPort}
                onChange={setNewTargetPort}
                min={1}
                max={65535}
                placeholder="5173"
                disabled={creating}
                className="min-w-0 flex-1 basis-40"
              />
              <button
                type="submit"
                disabled={creating || newTargetPort < 1 || newTargetPort > 65535}
                className={cn(
                  'inline-flex items-center justify-center gap-2 px-4 py-1.5 rounded-standard text-xs font-semibold shrink-0',
                  'bg-amber-gold text-text-inverse border border-amber-gold',
                  'hover:opacity-90 transition-opacity duration-vt-base',
                  'disabled:opacity-50 disabled:cursor-not-allowed',
                )}
              >
                {creating ? (
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                ) : (
                  <Plus className="w-3.5 h-3.5" />
                )}
                {t('portForward.add')}
              </button>
            </div>
          </FormField>
        </form>
        <PortForwardList forwards={data.forwards} onStop={setStopTarget} />
      </div>

      <ConfirmDialog
        open={stopTarget !== null}
        title={t('portForward.stopTitle')}
        message={t('portForward.stopMessage', { port: stopTarget ?? 0 })}
        confirmLabel={t('portForward.stop')}
        variant="danger"
        onConfirm={() => stopTarget !== null && handleStop(stopTarget)}
        onCancel={() => setStopTarget(null)}
      />

      {(saving || saveError) && (
        <div className={`text-xs ${saveError ? 'text-vt-error' : 'text-warm-khaki'}`}>
          {saveError ?? t('common.saving')}
        </div>
      )}
    </div>
  );
}
