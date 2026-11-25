import { useCallback, useEffect, useState } from 'react';
import { AlertCircle, ExternalLink, Loader2, RefreshCw, Square } from 'lucide-react';
import { getServerConfig, updateServerConfig } from '@/api/server';
import { getStaticHttpServices, stopStaticHttpService } from '@/api/static-http';
import { useConfigSection } from '@/hooks/use-config-section';
import { useToast } from '@/components/shared/Toast';
import { FormField } from '@/components/shared/FormField';
import { NumberInput } from '@/components/shared/NumberInput';
import { SelectField } from '@/components/shared/SelectField';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { useTranslation } from '@/i18n';
import { formatPath } from '@/utils/path';
import type { ServerConfig, StaticHttpService } from '@/types';

const SERVICES_POLL_MS = 5000;

export function StaticHttpSection() {
  const { addToast } = useToast();
  const { t } = useTranslation();
  const { data, loading, saving, loadError, saveError, updateField } =
    useConfigSection<ServerConfig>(getServerConfig, updateServerConfig);

  const [services, setServices] = useState<StaticHttpService[]>([]);
  const [servicesLoading, setServicesLoading] = useState(true);
  const [servicesError, setServicesError] = useState<string | null>(null);
  const [stoppingRoot, setStoppingRoot] = useState<string | null>(null);

  const refreshServices = useCallback(async () => {
    try {
      setServices(await getStaticHttpServices());
      setServicesError(null);
    } catch (error) {
      setServicesError(error instanceof Error ? error.message : String(error));
    } finally {
      setServicesLoading(false);
    }
  }, []);

  useEffect(() => {
    refreshServices();
    const timer = setInterval(refreshServices, SERVICES_POLL_MS);
    return () => clearInterval(timer);
  }, [refreshServices]);

  const handleStop = async (service: StaticHttpService) => {
    setStoppingRoot(service.root);
    try {
      const { stopped } = await stopStaticHttpService(service.root);
      addToast(
        stopped ? 'success' : 'error',
        t(stopped ? 'staticHttp.stopped' : 'staticHttp.stopFailed', { name: service.root }),
      );
    } catch (error) {
      addToast('error', error instanceof Error ? error.message : String(error));
    } finally {
      setStoppingRoot(null);
      refreshServices();
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
        <span className="font-light">{loadError ?? t('staticHttp.loadError')}</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SectionHeader title={t('staticHttp.title')} description={t('staticHttp.description')} />

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <FormField label={t('staticHttp.portRange')} description={t('staticHttp.portRangeDesc')}>
          <div className="grid grid-cols-2 gap-3">
            <NumberInput
              value={data.staticHttpPortStart}
              onChange={(v) => updateField('staticHttpPortStart', v)}
              min={1}
              max={65535}
            />
            <NumberInput
              value={data.staticHttpPortEnd}
              onChange={(v) => updateField('staticHttpPortEnd', v)}
              min={1}
              max={65535}
            />
          </div>
        </FormField>

        <FormField label={t('staticHttp.bindAddress')} description={t('staticHttp.bindAddressDesc')}>
          <SelectField
            value={data.staticHttpBindAddress}
            onChange={(e) => updateField('staticHttpBindAddress', e.target.value as 'local' | 'any')}
            options={[
              { value: 'local', label: 'Local' },
              { value: 'any', label: 'Any Address' },
            ]}
          />
        </FormField>

        {(saving || saveError) && (
          <div className={`text-xs ${saveError ? 'text-vt-error' : 'text-warm-khaki'}`}>
            {saveError ?? t('common.saving')}
          </div>
        )}
      </div>

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-4 border border-golden-sand">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-warm-black">
            {t('staticHttp.services', { count: services.length })}
          </h3>
          <button
            type="button"
            onClick={refreshServices}
            className="p-1.5 rounded-standard text-deep-olive hover:text-warm-black hover:bg-pale-sand transition-colors duration-vt-base"
            aria-label={t('common.refresh')}
            title={t('common.refresh')}
          >
            <RefreshCw className="w-4 h-4" />
          </button>
        </div>

        {servicesLoading ? (
          <div className="flex items-center justify-center py-6">
            <Loader2 className="w-5 h-5 animate-spin text-warm-khaki" />
          </div>
        ) : servicesError ? (
          <div className="flex items-center gap-2 text-xs text-vt-error py-4">
            <AlertCircle className="w-4 h-4 shrink-0" />
            <span>{servicesError}</span>
          </div>
        ) : services.length === 0 ? (
          <p className="text-xs text-warm-khaki py-4">{t('staticHttp.empty')}</p>
        ) : (
          <ul className="space-y-2">
            {services.map((service) => (
              <li
                key={`${service.root}-${service.port}`}
                className="flex items-center gap-3 p-3 rounded-comfortable border border-golden-sand bg-warm-ivory"
              >
                <div className="flex-1 min-w-0">
                  <div className="text-xs text-warm-black font-mono truncate" title={formatPath(service.root)}>
                    {formatPath(service.root)}
                  </div>
                  <div className="flex items-center gap-2 mt-1">
                    <span className="text-2xs font-medium text-deep-olive bg-pale-sand rounded-pill px-2 py-0.5">
                      :{service.port}
                    </span>
                    <a
                      href={service.url}
                      target="_blank"
                      rel="noreferrer"
                      className="inline-flex items-center gap-1 text-2xs text-accent-text hover:underline"
                    >
                      {service.url}
                      <ExternalLink className="w-3 h-3 shrink-0" />
                    </a>
                  </div>
                </div>
                <button
                  type="button"
                  onClick={() => handleStop(service)}
                  disabled={stoppingRoot === service.root}
                  className="inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-standard text-xs font-medium text-deep-olive hover:text-vt-error hover:bg-pale-sand disabled:opacity-50 transition-colors duration-vt-base whitespace-nowrap"
                  title={t('staticHttp.stop')}
                >
                  {stoppingRoot === service.root ? (
                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  ) : (
                    <Square className="w-3 h-3 fill-current" />
                  )}
                  {t('staticHttp.stop')}
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
