import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  AlertCircle,
  Loader2,
  ChevronDown,
  ChevronUp,
  Copy,
  ClipboardPaste,
} from 'lucide-react';
import { getLlmConfig, updateLlmConfig } from '@/api/llm';
import { getAddons } from '@/api/addon';
import { FormField } from '@/components/shared/FormField';
import { NumberInput } from '@/components/shared/NumberInput';
import { SelectField } from '@/components/shared/SelectField';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { ToggleField } from '@/components/shared/ToggleField';
import { useConfigSection } from '@/hooks/use-config-section';
import { useTranslation } from '@/i18n';
import { renderAddonConfigField, resolveAddonConfig } from '@/lib/addon-config';
import type { LlmConfig, LlmProviderConfig, TerminalLlmConfig, GitLlmConfig, CodeAnalysisConfig, TranslationConfig, AddonInfo } from '@/types';

type FeatureKey = 'terminal' | 'git' | 'gitConflict' | 'voicePolish' | 'codeAnalysis' | 'translation';

interface FeatureConfig {
  key: FeatureKey;
  titleKey: string;
  descKey: string;
  enableKey: string;
}

const FEATURES: FeatureConfig[] = [
  { key: 'terminal', titleKey: 'llm.terminal', descKey: 'llm.terminalDesc', enableKey: 'llm.terminalEnable' },
  { key: 'git', titleKey: 'llm.git', descKey: 'llm.gitDesc', enableKey: 'llm.gitEnable' },
  { key: 'gitConflict', titleKey: 'llm.gitConflict', descKey: 'llm.gitConflictDesc', enableKey: 'llm.gitConflictEnable' },
  { key: 'voicePolish', titleKey: 'llm.voicePolish', descKey: 'llm.voicePolishDesc', enableKey: 'llm.voicePolishEnable' },
  // Code Analysis 暂未启用，先在 UI 中隐藏（保留配置与下方渲染分支，便于后续恢复）
  // { key: 'codeAnalysis', titleKey: 'llm.codeAnalysis', descKey: 'llm.codeAnalysisDesc', enableKey: 'llm.codeAnalysisEnable' },
  { key: 'translation', titleKey: 'llm.translation', descKey: 'llm.translationDesc', enableKey: 'llm.translationEnable' },
];

// Keep in sync with the locales supported by the Header language switcher
// (`Locale` / `localeNames` in @/i18n) — same order, full native names as labels.
const LANGUAGE_OPTIONS = [
  { value: 'en', label: 'English' },
  { value: 'zh-CN', label: '简体中文' },
  { value: 'zh-TW', label: '繁體中文' },
  { value: 'de', label: 'Deutsch' },
  { value: 'es', label: 'Español' },
  { value: 'fr', label: 'Français' },
  { value: 'ja', label: '日本語' },
  { value: 'ko', label: '한국어' },
  { value: 'nl', label: 'Nederlands' },
  { value: 'pl', label: 'Polski' },
  { value: 'pt', label: 'Português' },
  { value: 'ru', label: 'Русский' },
  { value: 'th', label: 'ไทย' },
  { value: 'vi', label: 'Tiếng Việt' },
  { value: 'ar', label: 'العربية' },
  { value: 'id', label: 'Bahasa Indonesia' },
  { value: 'ms', label: 'Bahasa Melayu' },
  { value: 'uk', label: 'Українська' },
];

type FeaturePatch = Partial<LlmProviderConfig> & { outputLanguage?: string };

interface ClipboardEntry {
  addonId: string;
  config: Record<string, string>;
}

function FeatureCard({
  feature,
  providerConfig,
  llmAddons,
  expanded,
  onToggleExpand,
  onUpdate,
  clipboard,
  onCopy,
  onPaste,
  t,
}: {
  feature: FeatureConfig;
  providerConfig: LlmProviderConfig;
  llmAddons: AddonInfo[];
  expanded: boolean;
  onToggleExpand: () => void;
  onUpdate: (patch: FeaturePatch) => void;
  clipboard: ClipboardEntry | null;
  onCopy: (addonId: string, config: Record<string, string>) => void;
  onPaste: (config: Record<string, string>) => void;
  t: (key: string) => string;
}) {
  const enabledAddons = useMemo(
    () => llmAddons.filter((a) => a.enabled),
    [llmAddons],
  );

  const selectedAddon = useMemo(
    () => enabledAddons.find((a) => a.id === providerConfig.provider) ?? null,
    [enabledAddons, providerConfig.provider],
  );

  const providerOptions = useMemo(() => {
    if (enabledAddons.length === 0) {
      return [{ value: '', label: t('llm.noProviders') }];
    }
    return enabledAddons.map((a) => ({ value: a.id, label: a.name }));
  }, [enabledAddons, t]);

  const resolvedConfig = useMemo(
    () => selectedAddon ? resolveAddonConfig(selectedAddon, providerConfig.config) : {},
    [selectedAddon, providerConfig.config],
  );

  // Auto-detect option is only offered for terminal/git (their prompts leave
  // language to the model when set to "auto"). Code analysis/translation keep
  // their explicit language list.
  const languageOptionsWithAuto = useMemo(
    () => [{ value: 'auto', label: t('llm.languageAuto') }, ...LANGUAGE_OPTIONS],
    [t],
  );

  const handleConfigChange = useCallback(
    (key: string, value: string) => {
      onUpdate({ config: { ...providerConfig.config, [key]: value } });
    },
    [onUpdate, providerConfig.config],
  );

  return (
    <div className="rounded-comfortable border border-golden-sand bg-pale-sand/40">
      {/* Header row */}
      <div className="flex items-center justify-between p-4">
        <div className="flex items-center gap-3 flex-1 min-w-0">
          <button
            type="button"
            onClick={onToggleExpand}
            className="p-1 rounded-standard text-deep-olive hover:text-warm-black hover:bg-golden-cream transition-colors duration-vt-base shrink-0"
            aria-label={expanded ? 'Collapse' : 'Expand'}
          >
            {expanded ? (
              <ChevronUp className="w-4 h-4" />
            ) : (
              <ChevronDown className="w-4 h-4" />
            )}
          </button>
          <div className="flex-1 min-w-0">
            <h4 className="text-sm font-semibold text-warm-black truncate">
              {t(feature.titleKey)}
            </h4>
            <p className="text-xs text-deep-olive mt-0.5 truncate">
              {t(feature.descKey)}
            </p>
          </div>
        </div>
        <ToggleField
          enabled={providerConfig.enabled}
          onChange={(enabled) => onUpdate({ enabled })}
        />
      </div>

      {/* Expanded content — configurable even while the feature is disabled;
          the server only applies the config once the feature is enabled. */}
      {expanded && (
        <div className="px-4 pb-4 pt-0 space-y-5 border-t border-golden-sand/60 mt-0 pt-4">
          {!providerConfig.enabled && (
            <p className="text-xs text-deep-olive/80">{t('llm.disabledHint')}</p>
          )}
          <FormField label={t('llm.provider')}>
            {/* Display-only normalization: a persisted provider whose addon is
                disabled shows the placeholder instead of its raw id; enabling
                the feature re-picks a valid provider (see handleUpdate). */}
            <SelectField
              value={providerOptions.some((o) => o.value === providerConfig.provider) ? providerConfig.provider : ''}
              onChange={(e) => onUpdate({ provider: e.target.value })}
              options={providerOptions}
            />
          </FormField>

          {selectedAddon && selectedAddon.configFields.length > 0 && (
            <div>
              <div className="flex items-center justify-between mb-4">
                <h4 className="text-xs font-semibold text-warm-black">
                  {t('llm.providerConfig')}
                </h4>
                <div className="flex items-center gap-1">
                  <button
                    type="button"
                    onClick={() => onCopy(selectedAddon.id, resolvedConfig)}
                    className="p-1.5 rounded-standard text-deep-olive hover:text-warm-black hover:bg-golden-cream transition-colors duration-vt-base"
                    title={t('llm.copyConfig')}
                  >
                    <Copy className="w-3.5 h-3.5" />
                  </button>
                  {clipboard && clipboard.addonId === selectedAddon.id && (
                    <button
                      type="button"
                      onClick={() => onPaste(clipboard.config)}
                      className="p-1.5 rounded-standard text-deep-olive hover:text-warm-black hover:bg-golden-cream transition-colors duration-vt-base"
                      title={t('llm.pasteConfig')}
                    >
                      <ClipboardPaste className="w-3.5 h-3.5" />
                    </button>
                  )}
                </div>
              </div>
              <div className="space-y-4">
                {selectedAddon.configFields.map((field) => (
                  <FormField key={field.key} label={field.label} description={field.description}>
                    {renderAddonConfigField(
                      field,
                      resolvedConfig[field.key] ?? '',
                      (value) => handleConfigChange(field.key, value),
                      false,
                    )}
                  </FormField>
                ))}
              </div>
            </div>
          )}

          {feature.key === 'terminal' && (
            <>
              <FormField label={t('llm.maxFailures')} description={t('llm.maxFailuresDesc')}>
                <NumberInput
                  value={providerConfig.maxFailures ?? 3}
                  onChange={(v) => onUpdate({ maxFailures: Math.max(3, v) })}
                  min={3}
                />
              </FormField>
              <FormField label={t('llm.outputLanguage')} description={t('llm.terminalOutputLanguageDesc')}>
                <SelectField
                  value={(providerConfig as TerminalLlmConfig).outputLanguage ?? 'auto'}
                  onChange={(e) => onUpdate({ outputLanguage: e.target.value })}
                  options={languageOptionsWithAuto}
                />
              </FormField>
            </>
          )}

          {feature.key === 'git' && (
            <FormField label={t('llm.outputLanguage')} description={t('llm.gitOutputLanguageDesc')}>
              <SelectField
                value={(providerConfig as GitLlmConfig).outputLanguage ?? 'auto'}
                onChange={(e) => onUpdate({ outputLanguage: e.target.value })}
                options={languageOptionsWithAuto}
              />
            </FormField>
          )}

          {feature.key === 'gitConflict' && (
            <FormField label={t('llm.outputLanguage')} description={t('llm.gitConflictOutputLanguageDesc')}>
              <SelectField
                value={(providerConfig as GitLlmConfig).outputLanguage ?? 'auto'}
                onChange={(e) => onUpdate({ outputLanguage: e.target.value })}
                options={languageOptionsWithAuto}
              />
            </FormField>
          )}

          {feature.key === 'codeAnalysis' && (
            <FormField label={t('llm.outputLanguage')} description={t('llm.outputLanguageDesc')}>
              <SelectField
                value={(providerConfig as CodeAnalysisConfig).outputLanguage ?? 'en'}
                onChange={(e) => onUpdate({ outputLanguage: e.target.value })}
                options={LANGUAGE_OPTIONS}
              />
            </FormField>
          )}

          {feature.key === 'translation' && (
            <FormField label={t('llm.translationOutputLanguage')} description={t('llm.translationOutputLanguageDesc')}>
              <SelectField
                value={(providerConfig as TranslationConfig).outputLanguage ?? 'en'}
                onChange={(e) => onUpdate({ outputLanguage: e.target.value })}
                options={LANGUAGE_OPTIONS}
              />
            </FormField>
          )}
        </div>
      )}
    </div>
  );
}

export function LlmSection() {
  const { t } = useTranslation();
  const [llmAddons, setLlmAddons] = useState<AddonInfo[]>([]);
  const [expandedKey, setExpandedKey] = useState<FeatureKey | null>(null);
  const [clipboard, setClipboard] = useState<ClipboardEntry | null>(null);

  const fetchFn = useCallback(() => getLlmConfig(), []);
  const saveFn = useCallback((config: LlmConfig) => updateLlmConfig(config), []);
  const { data, loading, saving, loadError, saveError, updateData } =
    useConfigSection<LlmConfig>(fetchFn, saveFn);

  // Load LLM addons on mount (all LLM type, not filtered by enabled)
  useEffect(() => {
    let cancelled = false;
    getAddons()
      .then((addons) => {
        if (!cancelled) {
          setLlmAddons(addons.filter((a) => a.addonType === 'llm'));
        }
      })
      .catch(() => {
        // Non-critical: provider list will be empty
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const enabledLlmAddons = useMemo(
    () => llmAddons.filter((a) => a.enabled),
    [llmAddons],
  );

  const handleUpdate = useCallback(
    (feature: FeatureKey, patch: FeaturePatch) => {
      if (!data) return;
      const current = data[feature];
      if (!current) return;

      // When enabling, pick first valid provider if current one is invalid
      let provider = patch.provider ?? current.provider;
      if (patch.enabled === true) {
        const isValid = provider && enabledLlmAddons.some((a) => a.id === provider);
        if (!isValid) {
          provider = enabledLlmAddons[0]?.id ?? '';
        }
      }

      updateData({
        [feature]: {
          ...current,
          ...patch,
          provider,
        },
      });
    },
    [data, enabledLlmAddons, updateData],
  );

  const handleToggleExpand = useCallback((key: FeatureKey) => {
    setExpandedKey((prev) => (prev === key ? null : key));
  }, []);

  const handleCopy = useCallback((addonId: string, config: Record<string, string>) => {
    setClipboard({ addonId, config });
  }, []);

  const handlePaste = useCallback((feature: FeatureKey, config: Record<string, string>) => {
    if (!data) return;
    const current = data[feature];
    if (!current) return;
    updateData({
      [feature]: {
        ...current,
        config: { ...current.config, ...config },
      },
    });
  }, [data, updateData]);

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
        <span className="font-light">{loadError ?? t('llm.loadError')}</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SectionHeader title={t('llm.title')} description={t('llm.description')} />

      <div className="space-y-4">
        {FEATURES.filter((f) => data[f.key] != null).map((feature) => (
          <FeatureCard
            key={feature.key}
            feature={feature}
            providerConfig={data[feature.key]!}
            llmAddons={llmAddons}
            expanded={expandedKey === feature.key}
            onToggleExpand={() => handleToggleExpand(feature.key)}
            onUpdate={(patch) => handleUpdate(feature.key, patch)}
            clipboard={clipboard}
            onCopy={handleCopy}
            onPaste={(config) => handlePaste(feature.key, config)}
            t={t}
          />
        ))}
      </div>

      {(saving || saveError) && (
        <div className={`text-xs ${saveError ? 'text-vt-error' : 'text-warm-khaki'}`}>
          {saveError ?? t('common.saving')}
        </div>
      )}
    </div>
  );
}
