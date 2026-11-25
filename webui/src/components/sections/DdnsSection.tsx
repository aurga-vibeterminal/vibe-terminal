import { useCallback, useEffect, useMemo, useState } from 'react';
import { Loader2, AlertCircle, RefreshCw } from 'lucide-react';
import { getDdnsConfig, updateDdnsConfig, triggerDdnsSync, getDdnsNetworkInterfaces } from '@/api/ddns';
import { getAddons } from '@/api/addon';
import { useConfigSection } from '@/hooks/use-config-section';
import { useToast } from '@/components/shared/Toast';
import { FormField } from '@/components/shared/FormField';
import { SelectField } from '@/components/shared/SelectField';
import { NumberInput } from '@/components/shared/NumberInput';
import { ToggleField } from '@/components/shared/ToggleField';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { StatusBadge } from '@/components/shared/StatusBadge';
import { useTranslation } from '@/i18n';
import { renderAddonConfigField, resolveAddonConfig } from '@/lib/addon-config';
import type { DdnsConfig, ServerNetworkInterface, AddonInfo } from '@/types';

type DdnsIpFamily = 'ipv4' | 'ipv6' | 'dual';

function getAvailableFamilies(networkInterface: ServerNetworkInterface): DdnsIpFamily[] {
  const families: DdnsIpFamily[] = [];
  if (networkInterface.ipv4Addresses.length > 0 && networkInterface.ipv6Addresses.length > 0) {
    families.push('dual');
  }
  if (networkInterface.ipv4Addresses.length > 0) {
    families.push('ipv4');
  }
  if (networkInterface.ipv6Addresses.length > 0) {
    families.push('ipv6');
  }
  return families;
}

function getAddressForFamily(networkInterface: ServerNetworkInterface, ipFamily: DdnsIpFamily): string {
  switch (ipFamily) {
    case 'ipv6':
      return networkInterface.ipv6Addresses[0] ?? '';
    case 'dual':
      return (networkInterface.ipv4Addresses[0] ?? '') + ' / ' + (networkInterface.ipv6Addresses[0] ?? '');
    case 'ipv4':
    default:
      return networkInterface.ipv4Addresses[0] ?? '';
  }
}

function resolveConfig(addon: AddonInfo, saved: Record<string, string>): Record<string, string> {
  // Delegate to the shared helper so empty-string saved values fall back to
  // the manifest default (matches LLM/Voice sections).
  return resolveAddonConfig(addon, saved);
}

export function DdnsSection() {
  const { addToast } = useToast();
  const { t } = useTranslation();
  const [syncing, setSyncing] = useState(false);
  const [ddnsAddons, setDdnsAddons] = useState<AddonInfo[]>([]);

  // Network interface state
  const [networkInterfaces, setNetworkInterfaces] = useState<ServerNetworkInterface[]>([]);
  const [selectedInterfaceId, setSelectedInterfaceId] = useState('');

  const fetchFn = useCallback(() => getDdnsConfig(), []);
  const saveFn = useCallback((data: DdnsConfig) => updateDdnsConfig(data), []);

  const { data, loading, saving, loadError, saveError, updateField, updateData, save } =
    useConfigSection<DdnsConfig>(fetchFn, saveFn, { autoSave: false });

  // Load DDNS addons on mount
  useEffect(() => {
    let cancelled = false;
    getAddons()
      .then((addons) => {
        if (!cancelled) {
          const filtered = addons.filter((a) => a.addonType === 'ddns');
          setDdnsAddons(filtered);
        }
      })
      .catch(() => {
        // Non-critical: provider list will be empty
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Load network interfaces on mount
  useEffect(() => {
    let cancelled = false;
    getDdnsNetworkInterfaces()
      .then((interfaces) => {
        if (!cancelled) {
          setNetworkInterfaces(interfaces);
        }
      })
      .catch(() => {
        // Non-critical: interface list will be empty
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Restore saved interface from config, or auto-select first interface when interfaces load
  useEffect(() => {
    if (networkInterfaces.length === 0) {
      if (selectedInterfaceId) {
        setSelectedInterfaceId('');
      }
      return;
    }

    const savedId = data?.networkInterface;
    if (savedId && networkInterfaces.some((ni) => ni.id === savedId)) {
      if (selectedInterfaceId !== savedId) {
        setSelectedInterfaceId(savedId);
      }
      return;
    }

    if (!networkInterfaces.some((ni) => ni.id === selectedInterfaceId)) {
      setSelectedInterfaceId(networkInterfaces[0].id);
    }
  }, [networkInterfaces, selectedInterfaceId, data?.networkInterface]);

  // Sync selectedInterfaceId to config when user changes it
  const handleInterfaceChange = useCallback((id: string) => {
    setSelectedInterfaceId(id);
    updateField('networkInterface', id);
  }, [updateField]);

  const selectedInterface = useMemo(
    () =>
      networkInterfaces.find((ni) => ni.id === selectedInterfaceId) ?? networkInterfaces[0],
    [networkInterfaces, selectedInterfaceId],
  );

  // Auto-select first available IP family only when unset or invalid
  useEffect(() => {
    if (!selectedInterface || !data) {
      return;
    }

    const allFamilies: DdnsIpFamily[] = ['ipv4', 'ipv6', 'dual'];
    const currentFamily = data.ipFamily as DdnsIpFamily;
    const isValid = allFamilies.includes(currentFamily);
    if (!isValid) {
      const available = getAvailableFamilies(selectedInterface);
      if (available.length > 0) {
        updateField('ipFamily', available[0]);
      }
    }
  }, [selectedInterface, data]);

  const selectedIpAddress = useMemo(
    () => (selectedInterface ? getAddressForFamily(selectedInterface, (data?.ipFamily ?? 'ipv4') as DdnsIpFamily) : ''),
    [selectedInterface, data?.ipFamily],
  );

  // Only enabled addons are selectable as the provider — a disabled plugin
  // must not be offered (mirrors LlmSection).
  const enabledDdnsAddons = useMemo(
    () => ddnsAddons.filter((a) => a.enabled),
    [ddnsAddons],
  );

  const selectedAddon = useMemo(
    () => enabledDdnsAddons.find((a) => a.id === data?.provider) ?? null,
    [enabledDdnsAddons, data?.provider],
  );

  const resolvedConfig = useMemo(
    () => selectedAddon ? resolveConfig(selectedAddon, data?.config ?? {}) : {},
    [selectedAddon, data?.config],
  );

  const validationErrors = useMemo(() => {
    if (!selectedAddon) return {};
    const errors: Record<string, string> = {};
    for (const field of selectedAddon.configFields) {
      if (field.required && !(resolvedConfig[field.key] ?? '').trim()) {
        errors[field.key] = t('ddns.fieldRequired', { field: field.label });
      }
    }
    return errors;
  }, [selectedAddon, resolvedConfig, t]);

  const hasValidationErrors = Object.keys(validationErrors).length > 0;

  const providerOptions = useMemo(() => {
    if (enabledDdnsAddons.length > 0) {
      return enabledDdnsAddons.map((a) => ({ value: a.id, label: a.name }));
    }
    return [{ value: '', label: t('ddns.noProviders') }];
  }, [enabledDdnsAddons, t]);

  const handleAddonConfigChange = useCallback((key: string, value: string) => {
    updateData({ config: { ...(data?.config ?? {}), [key]: value } });
  }, [data?.config, updateData]);

  const handleProviderChange = useCallback(
    (provider: string) => {
      const addon = enabledDdnsAddons.find((a) => a.id === provider);
      updateData({
        provider,
        config: addon ? resolveConfig(addon, addon.userConfig) : {},
      });
    },
    [enabledDdnsAddons, updateData],
  );

  const handleEnabledChange = useCallback(
    (enabled: boolean) => {
      if (!enabled) {
        updateData({ enabled });
        return;
      }

      const currentProvider = data?.provider ?? '';
      const hasCurrentProvider = enabledDdnsAddons.some((a) => a.id === currentProvider);
      if (hasCurrentProvider) {
        updateData({ enabled });
        return;
      }

      const addon = enabledDdnsAddons[0];
      updateData({
        enabled,
        provider: addon?.id ?? currentProvider,
        config: addon ? resolveConfig(addon, addon.userConfig) : data?.config ?? {},
      });
    },
    [data?.config, data?.provider, enabledDdnsAddons, updateData],
  );

  const handleSave = async () => {
    // Required fields only matter once the service runs; a disabled service
    // may keep a half-configured provider saved without executing it.
    if (data?.enabled && hasValidationErrors) {
      addToast('error', t('ddns.fixErrors'));
      return;
    }
    const success = await save();
    if (success) {
      addToast('success', t('ddns.saved'));
    }
  };

  const handleSync = async () => {
    setSyncing(true);
    try {
      const saved = await save();
      if (!saved) {
        return;
      }
      const result = await triggerDdnsSync();
      updateField('syncStatus', result.syncStatus);
      addToast('success', t('ddns.synced', { ip: result.syncStatus.lastIp ?? '\u2014' }));
    } catch {
      addToast('error', t('ddns.syncFailed'));
    } finally {
      setSyncing(false);
    }
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
        <span className="font-light">{loadError ?? t('ddns.loadError')}</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SectionHeader title={t('ddns.title')} description={t('ddns.description')} />

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <FormField label={t('ddns.service')}>
          <ToggleField
            enabled={data.enabled}
            onChange={handleEnabledChange}
            label={t('ddns.enable')}
          />
        </FormField>

        <FormField label={t('ddns.provider')}>
          {/* Editable while the service is off — the config is saved but the
              DDNS addon only runs once the service is enabled. Display-only
              normalization: a persisted provider whose addon is disabled
              falls back to the placeholder option instead of its raw id. */}
          <SelectField
            value={providerOptions.some((o) => o.value === data.provider) ? data.provider : ''}
            onChange={(e) => handleProviderChange(e.target.value)}
            options={providerOptions}
          />
        </FormField>

        <FormField label={t('ddns.pollingInterval')}>
          <NumberInput
            value={data.pollingIntervalMinutes}
            onChange={(v) => updateField('pollingIntervalMinutes', v)}
            min={1}
            max={1440}
          />
        </FormField>

        {/* Dynamic addon config fields */}
        {selectedAddon && selectedAddon.configFields.length > 0 && (
          <>
            <div className="border-t border-golden-sand/60 pt-4">
              <h4 className="text-xs font-semibold text-warm-black mb-1">
                {t('ddns.providerConfig')}
              </h4>
              {data.ddnsDomain && (
                <p className="text-xs font-normal text-warm-khaki mb-4">
                  {t('ddns.fullDomain')}: <span className="font-mono text-warm-black">{data.ddnsDomain}</span>
                </p>
              )}
              <div className="space-y-3">
                {selectedAddon.configFields.map((field) => (
                  <FormField key={field.key} label={field.label} description={field.description} error={validationErrors[field.key]}>
                    {renderAddonConfigField(
                      field,
                      resolvedConfig[field.key] ?? '',
                      (value) => handleAddonConfigChange(field.key, value),
                      false,
                    )}
                  </FormField>
                ))}
              </div>
            </div>
          </>
        )}

        {/* Save button (manual save only - no auto-save) */}
        <div className="border-t border-golden-sand/60 pt-4 flex items-center gap-3">
          <button
            type="button"
            onClick={() => void handleSave()}
            disabled={saving || (data.enabled && hasValidationErrors)}
            className="inline-flex items-center gap-2 px-4 py-1.5 bg-amber-gold text-text-inverse rounded-standard font-medium text-xs hover:opacity-90 transition-opacity disabled:opacity-50 border border-amber-gold"
          >
            {saving ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : null}
            {t('ddns.save')}
          </button>
          {hasValidationErrors && data.enabled && (
            <span className="text-xs text-vt-error">{t('ddns.fixErrors')}</span>
          )}
        </div>
      </div>

      {/* Network interface & IP address */}
      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <h3 className="text-sm font-semibold text-warm-black tracking-wide">{t('ddns.networkInterface')}</h3>

        <FormField label={t('ddns.interface')}>
          <SelectField
            value={selectedInterface?.id ?? ''}
            onChange={(e) => handleInterfaceChange(e.target.value)}
            disabled={networkInterfaces.length === 0}
            options={
              networkInterfaces.length === 0
                ? [{ value: '', label: t('ddns.noInterfaces') }]
                : networkInterfaces.map((ni) => ({ value: ni.id, label: ni.name }))
            }
          />
        </FormField>

        <FormField label={t('ddns.ipFamily')}>
          <SelectField
            value={data.ipFamily}
            onChange={(e) => updateField('ipFamily', e.target.value)}
            disabled={!selectedInterface}
            options={[
              {
                value: 'dual',
                label: t('ddns.dualStack'),
                disabled:
                  !selectedInterface ||
                  selectedInterface.ipv4Addresses.length === 0 ||
                  selectedInterface.ipv6Addresses.length === 0,
              },
              {
                value: 'ipv4',
                label: t('ddns.ipv4'),
                disabled: !selectedInterface || selectedInterface.ipv4Addresses.length === 0,
              },
              {
                value: 'ipv6',
                label: t('ddns.ipv6'),
                disabled: !selectedInterface || selectedInterface.ipv6Addresses.length === 0,
              },
            ]}
          />
        </FormField>

        <FormField label={t('ddns.ipAddress')}>
          <div className="px-2 py-1.5 bg-bg-inset border border-golden-sand rounded-standard text-warm-black text-xs font-mono min-h-[30px] flex items-center">
            {selectedIpAddress || (networkInterfaces.length === 0 ? t('ddns.noInterfaces') : t('ddns.noAddressForFamily'))}
          </div>
        </FormField>
      </div>

      {/* Sync status */}
      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-warm-black tracking-wide">{t('ddns.syncStatus')}</h3>
          <button
            type="button"
            onClick={() => void handleSync()}
            disabled={syncing || !data.enabled}
            className="inline-flex items-center gap-2 px-4 py-1.5 bg-amber-gold text-text-inverse rounded-standard font-medium text-xs hover:opacity-90 transition-opacity disabled:opacity-50 border border-amber-gold"
          >
            {syncing ? (
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
            ) : (
              <RefreshCw className="w-3.5 h-3.5" />
            )}
            {t('ddns.syncNow')}
          </button>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
          <div>
            <span className="text-xs font-medium text-warm-khaki">{t('ddns.status')}</span>
            <div className="mt-2">
              <StatusBadge
                status={data.syncStatus.syncing ? 'syncing' : data.enabled ? 'connected' : 'inactive'}
              />
            </div>
          </div>
          <div>
            <span className="text-xs font-medium text-warm-khaki">{t('ddns.lastIp')}</span>
            <div className="mt-2 text-warm-black font-mono text-xs">{data.syncStatus.lastIp ?? '\u2014'}</div>
          </div>
          <div>
            <span className="text-xs font-medium text-warm-khaki">{t('ddns.lastSync')}</span>
            <div className="mt-2 text-warm-black font-light text-xs">
              {data.syncStatus.lastSync ? new Date(data.syncStatus.lastSync).toLocaleString() : '\u2014'}
            </div>
          </div>
          <div>
            <span className="text-xs font-medium text-warm-khaki">{t('ddns.nextSync')}</span>
            <div className="mt-2 text-warm-black font-light text-xs">
              {data.syncStatus.nextSync ? new Date(data.syncStatus.nextSync).toLocaleString() : '\u2014'}
            </div>
          </div>
        </div>
      </div>

      {(saving || saveError) && (
        <div className={`text-xs ${saveError ? 'text-vt-error' : 'text-warm-khaki'}`}>
          {saveError ?? t('common.saving')}
        </div>
      )}
    </div>
  );
}
