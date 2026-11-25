import { useCallback, useState } from 'react';
import { AlertCircle, ChevronDown, ChevronRight, Loader2, Plus, Trash2 } from 'lucide-react';
import { getTerminalConfig, updateTerminalConfig } from '@/api';
import { FormField } from '@/components/shared/FormField';
import { NumberInput } from '@/components/shared/NumberInput';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { ToggleField } from '@/components/shared/ToggleField';
import { useConfigSection } from '@/hooks/use-config-section';
import { useTranslation } from '@/i18n';
import type { TerminalConfig, TerminalEnvVar } from '@/types';

export function TerminalSesion() {
  const { t } = useTranslation();
  const [envVarsOpen, setEnvVarsOpen] = useState(true);

  const fetchFn = useCallback(() => getTerminalConfig(), []);
  const saveFn = useCallback((config: TerminalConfig) => updateTerminalConfig(config), []);
  const { data, loading, saving, loadError, saveError, updateField } =
    useConfigSection<TerminalConfig>(fetchFn, saveFn);

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
        <span className="font-light">{loadError ?? t('terminal.loadError')}</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SectionHeader title={t('terminal.title')} description={t('terminal.description')} />

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <FormField label={t('terminal.monitoring')}>
          <ToggleField
            enabled={data.enabled}
            onChange={(value) => updateField('enabled', value)}
            label={t('terminal.monitoringEnable')}
          />
        </FormField>

        <FormField label={t('terminal.autoConfirm')} description={t('terminal.autoConfirmDesc')}>
          <ToggleField
            enabled={data.autoConfirm}
            onChange={(value) => updateField('autoConfirm', value)}
            label={t('terminal.autoConfirmEnable')}
            disabled={!data.enabled}
          />
        </FormField>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
          <FormField
            label={t('terminal.analysisDelay')}
            description={t('terminal.analysisDelayDesc')}
          >
            <NumberInput
              value={data.analysisDelaySeconds}
              onChange={(value) => updateField('analysisDelaySeconds', value)}
              min={1}
            />
          </FormField>

          <FormField label={t('terminal.confirmationGrace')}>
            <NumberInput
              value={data.confirmationGraceSeconds}
              onChange={(value) => updateField('confirmationGraceSeconds', value)}
              min={1}
            />
          </FormField>
        </div>

        <div className="flex items-start gap-2 rounded-comfortable border border-golden-sand bg-warm-white/50 p-3 text-sm text-warm-black">
          <AlertCircle className="mt-0.5 h-4 w-4 shrink-0" />
          <span className="font-light">{t('terminal.autoModeTip')}</span>
        </div>

        <FormField label={t('terminal.notifyOnCompletion')} description={t('terminal.notifyOnCompletionDesc')}>
          <ToggleField
            enabled={data.notifyOnCompletion}
            onChange={(value) => updateField('notifyOnCompletion', value)}
            label={t('terminal.notifyOnCompletionEnable')}
            disabled={!data.enabled}
          />
        </FormField>
      </div>

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <div className="flex items-center justify-between gap-4">
          <button
            type="button"
            onClick={() => setEnvVarsOpen((current) => !current)}
            className="flex items-center gap-2 text-left"
          >
            {envVarsOpen ? (
              <ChevronDown className="w-4 h-4 shrink-0 text-warm-black" />
            ) : (
              <ChevronRight className="w-4 h-4 shrink-0 text-warm-black" />
            )}
            <div>
              <h3 className="text-sm font-semibold text-warm-black tracking-wide">
                {t('terminal.envVars')}
              </h3>
              <p className="text-xs font-light text-warm-khaki">{t('terminal.envVarsDesc')}</p>
            </div>
          </button>
          <button
            type="button"
            onClick={() => {
              const next: TerminalEnvVar[] = [
                ...data.envVars,
                { name: '', value: '', enabled: true },
              ];
              updateField('envVars', next);
              setEnvVarsOpen(true);
            }}
            className="inline-flex items-center gap-2 px-3 py-1.5 rounded-standard border border-golden-sand text-xs font-medium text-warm-black hover:bg-pale-sand transition-colors duration-vt-base"
          >
            <Plus className="w-4 h-4" />
            {t('terminal.envVarAdd')}
          </button>
        </div>

        {envVarsOpen && (
          data.envVars.length === 0 ? (
            <p className="text-sm font-light text-warm-khaki">{t('terminal.envVarsEmpty')}</p>
          ) : (
            <div className="space-y-3">
              {data.envVars.map((envVar, index) => (
                <div key={index} className="flex items-center gap-3">
                  <ToggleField
                    enabled={envVar.enabled}
                    onChange={(value) => {
                      const next = data.envVars.map((item, itemIndex) =>
                        itemIndex === index ? { ...item, enabled: value } : item,
                      );
                      updateField('envVars', next);
                    }}
                  />
                  <input
                    type="text"
                    value={envVar.name}
                    onChange={(event) => {
                      const next = data.envVars.map((item, itemIndex) =>
                        itemIndex === index ? { ...item, name: event.target.value } : item,
                      );
                      updateField('envVars', next);
                    }}
                    placeholder={t('terminal.envVarNamePlaceholder')}
                    className="w-40 bg-bg-inset px-2 py-1.5 text-sm rounded-standard border border-golden-sand focus:outline-none focus:border-amber-gold text-warm-black"
                  />
                  <input
                    type="text"
                    value={envVar.value}
                    onChange={(event) => {
                      const next = data.envVars.map((item, itemIndex) =>
                        itemIndex === index ? { ...item, value: event.target.value } : item,
                      );
                      updateField('envVars', next);
                    }}
                    placeholder={t('terminal.envVarValuePlaceholder')}
                    className="w-full bg-bg-inset px-2 py-1.5 text-sm rounded-standard border border-golden-sand focus:outline-none focus:border-amber-gold text-warm-black"
                  />
                  <button
                    type="button"
                    aria-label={t('terminal.envVarDelete')}
                    onClick={() => {
                      const next = data.envVars.filter((_, itemIndex) => itemIndex !== index);
                      updateField('envVars', next);
                    }}
                    className="shrink-0 p-1.5 rounded-standard text-warm-khaki hover:text-vt-error hover:bg-pale-sand transition-colors duration-vt-base"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              ))}
            </div>
          )
        )}
      </div>

      {(saving || saveError) && (
        <div className={`text-xs ${saveError ? 'text-vt-error' : 'text-warm-khaki'}`}>
          {saveError ?? t('common.saving')}
        </div>
      )}
    </div>
  );
}
