import { useCallback, useEffect, useState } from 'react';
import { AlertCircle, ChevronDown, ChevronUp, Loader2, Pencil, Plus, RefreshCw, Trash2, X } from 'lucide-react';
import { detectUartSerialPorts, getUartConfig, updateUartConfig } from '@/api';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { FormField } from '@/components/shared/FormField';
import { NumberInput } from '@/components/shared/NumberInput';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { SelectField } from '@/components/shared/SelectField';
import { useToast } from '@/components/shared/Toast';
import { useConfigSection } from '@/hooks/use-config-section';
import { useTranslation } from '@/i18n';
import { terminalEncodingOptionsWithAuto } from '@/constants/terminal-encodings';
import type { UartConfig, UartFlowControl, UartParity, UartPortConfig, UartStopBits } from '@/types';

const inputClassName = [
  'w-full bg-bg-inset px-2 py-1.5 text-sm font-normal',
  'rounded-standard border border-golden-sand',
  'transition-colors duration-vt-base',
  'focus:outline-none focus:border-amber-gold',
  'disabled:opacity-50 disabled:cursor-not-allowed',
  'text-warm-black',
].join(' ');

function createNewPortDraft(initialPort = ''): Omit<UartPortConfig, 'id'> {
  return {
    name: '',
    port: initialPort,
    baudRate: 115200,
    dataBits: 8,
    stopBits: 'one',
    parity: 'none',
    flowControl: 'none',
    encoding: undefined,
  };
}

function createUartId() {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }

  return `uart-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function withCurrentPort(ports: string[], currentPort: string) {
  const values = currentPort ? [currentPort, ...ports] : ports;
  return Array.from(new Set(values.filter(Boolean)));
}

export function UartSection() {
  const { addToast } = useToast();
  const { t } = useTranslation();
  const [availablePorts, setAvailablePorts] = useState<string[]>([]);
  const [detectingPorts, setDetectingPorts] = useState(false);
  const [removeTarget, setRemoveTarget] = useState<string | null>(null);
  const [editingPortId, setEditingPortId] = useState<string | null>(null);
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [formPort, setFormPort] = useState<Omit<UartPortConfig, 'id'>>(() => createNewPortDraft());

  const fetchFn = useCallback(() => getUartConfig(), []);
  const saveFn = useCallback((config: UartConfig) => updateUartConfig(config), []);
  const { data, loading, saving, loadError, saveError, updateField } =
    useConfigSection<UartConfig>(fetchFn, saveFn);

  const parityOptions: { value: UartParity; label: string }[] = [
    { value: 'none', label: t('uart.parityNone') },
    { value: 'odd', label: t('uart.parityOdd') },
    { value: 'even', label: t('uart.parityEven') },
  ];
  const stopBitsOptions: { value: UartStopBits; label: string }[] = [
    { value: 'one', label: '1' },
    { value: 'two', label: '2' },
  ];
  const flowControlOptions: { value: UartFlowControl; label: string }[] = [
    { value: 'none', label: t('uart.flowControlNone') },
    { value: 'software', label: t('uart.flowControlSoftware') },
    { value: 'hardware', label: t('uart.flowControlHardware') },
  ];
  const dataBitsOptions = [5, 6, 7, 8].map((value) => ({
    value: String(value),
    label: String(value),
  }));
  const isEditing = editingPortId !== null;

  const refreshPorts = useCallback(async () => {
    setDetectingPorts(true);
    try {
      const ports = await detectUartSerialPorts();
      setAvailablePorts(ports);
      setFormPort((current) => ({
        ...current,
        port: current.port || ports[0] || '',
      }));
      addToast('info', t('uart.portsFound', { count: ports.length }));
    } catch {
      addToast('error', t('uart.detectFailed'));
    } finally {
      setDetectingPorts(false);
    }
  }, [addToast, t]);

  useEffect(() => {
    void refreshPorts();
  }, [refreshPorts]);

  const resetForm = useCallback(() => {
    setEditingPortId(null);
    setAdvancedOpen(false);
    setFormPort(createNewPortDraft(availablePorts[0] || ''));
  }, [availablePorts]);

  const handleEditPort = useCallback((port: UartPortConfig) => {
    setEditingPortId(port.id);
    setAdvancedOpen(false);
    setFormPort({
      name: port.name,
      port: port.port,
      baudRate: port.baudRate,
      dataBits: port.dataBits,
      stopBits: port.stopBits,
      parity: port.parity,
      flowControl: port.flowControl,
      encoding: port.encoding,
    });
  }, []);

  const handleSubmitPort = useCallback(() => {
    if (!data) {
      return;
    }

    const trimmedName = formPort.name.trim();
    const trimmedPort = formPort.port.trim();

    if (!trimmedName) {
      addToast('error', t('uart.nameRequired'));
      return;
    }

    if (!trimmedPort) {
      addToast('error', t('uart.portRequired'));
      return;
    }

    const nextPort = {
      ...formPort,
      name: trimmedName,
      port: trimmedPort,
    };

    if (editingPortId) {
      updateField(
        'ports',
        data.ports.map((port) =>
          port.id === editingPortId
            ? {
                ...port,
                ...nextPort,
              }
            : port,
        ),
      );
      addToast('success', t('uart.updated', { name: trimmedName }));
    } else {
      updateField('ports', [
        ...data.ports,
        {
          id: createUartId(),
          ...nextPort,
        },
      ]);
      addToast('success', t('uart.created', { name: trimmedName }));
    }

    resetForm();
  }, [addToast, data, editingPortId, formPort, resetForm, t, updateField]);

  const handleRemovePort = useCallback(() => {
    if (!data || !removeTarget) {
      return;
    }

    updateField(
      'ports',
      data.ports.filter((port) => port.id !== removeTarget),
    );
    if (editingPortId === removeTarget) {
      resetForm();
    }
    setRemoveTarget(null);
    addToast('success', t('uart.removed'));
  }, [addToast, data, editingPortId, removeTarget, resetForm, t, updateField]);

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
        <span className="font-light">{loadError ?? t('uart.loadError')}</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SectionHeader title="UART" description={t('uart.description')}>
        <button
          type="button"
          onClick={refreshPorts}
          disabled={detectingPorts}
          className="inline-flex items-center gap-2 px-4 py-1.5 bg-golden-cream text-warm-black rounded-standard font-medium text-xs hover:bg-pale-sand transition-colors duration-vt-base border border-golden-sand disabled:opacity-50"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${detectingPorts ? 'animate-spin' : ''}`} />
          {t('common.refresh')}
        </button>
      </SectionHeader>

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <div className="flex items-center justify-between gap-4">
          <h3 className="text-sm font-semibold text-warm-black tracking-wide">
            {t('uart.configuredPorts', { count: data.ports.length })}
          </h3>
          <div className="text-xs text-deep-olive">
            {availablePorts.length > 0
              ? t('uart.availablePorts', { count: availablePorts.length })
              : t('uart.noDetectedPorts')}
          </div>
        </div>

        {data.ports.length === 0 ? (
          <div className="rounded-comfortable border border-dashed border-golden-sand p-4 text-xs text-deep-olive">
            {t('uart.noConfiguredPorts')}
          </div>
        ) : (
          <div className="space-y-4">
            {data.ports.map((port) => (
              <div
                key={port.id}
                className="rounded-comfortable border border-golden-sand p-4 bg-pale-sand/40"
              >
                <div className="flex items-start justify-between gap-4">
                  <div className="space-y-1">
                    <h4 className="text-sm font-semibold text-warm-black">
                      {port.name || t('uart.unnamedPort')}
                    </h4>
                    <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-deep-olive">
                      <span>{t('uart.serialPortValue', { value: port.port })}</span>
                      <span>{t('uart.baudRateValue', { value: port.baudRate })}</span>
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      type="button"
                      onClick={() => handleEditPort(port)}
                      className="inline-flex items-center justify-center w-8 h-8 rounded-standard border border-golden-sand text-warm-black hover:bg-golden-cream transition-colors duration-vt-base"
                      title={t('uart.editPortAction')}
                      aria-label={t('uart.editPortAction')}
                    >
                      <Pencil className="w-4 h-4" />
                    </button>
                    <button
                      type="button"
                      onClick={() => setRemoveTarget(port.id)}
                      className="inline-flex items-center justify-center w-8 h-8 rounded-standard border border-vt-error text-vt-error hover:bg-vt-error hover:text-text-inverse transition-colors duration-vt-base"
                      title={t('common.remove')}
                      aria-label={t('common.remove')}
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <div className="flex items-center justify-between gap-4">
          <h3 className="text-sm font-semibold text-warm-black tracking-wide">
            {isEditing ? t('uart.editPort') : t('uart.newPort')}
          </h3>
          <button
            type="button"
            onClick={() => setAdvancedOpen((current) => !current)}
            className="inline-flex items-center gap-2 px-3 py-1.5 rounded-standard border border-golden-sand text-xs font-medium text-warm-black hover:bg-pale-sand transition-colors duration-vt-base"
          >
            {advancedOpen ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
            {advancedOpen ? t('uart.hideAdvanced') : t('uart.showAdvanced')}
          </button>
        </div>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
          <FormField label={t('uart.name')}>
            <input
              type="text"
              value={formPort.name}
              onChange={(event) => setFormPort((current) => ({ ...current, name: event.target.value }))}
              className={inputClassName}
              placeholder={t('uart.namePlaceholder')}
            />
          </FormField>
          <FormField label={t('uart.serialPort')}>
            {availablePorts.length > 0 ? (
              <SelectField
                value={formPort.port}
                onChange={(event) => setFormPort((current) => ({ ...current, port: event.target.value }))}
                options={withCurrentPort(availablePorts, formPort.port).map((value) => ({ value, label: value }))}
              />
            ) : (
              <input
                type="text"
                value={formPort.port}
                onChange={(event) => setFormPort((current) => ({ ...current, port: event.target.value }))}
                className={inputClassName}
                placeholder="COM3"
              />
            )}
          </FormField>
          <FormField label={t('uart.baudRate')}>
            <NumberInput
              value={formPort.baudRate}
              onChange={(value) => setFormPort((current) => ({ ...current, baudRate: value }))}
              min={1}
              className={inputClassName}
            />
          </FormField>
          {advancedOpen && (
            <>
              <FormField label={t('uart.dataBits')}>
                <SelectField
                  value={String(formPort.dataBits)}
                  onChange={(event) => setFormPort((current) => ({ ...current, dataBits: Number(event.target.value) }))}
                  options={dataBitsOptions}
                />
              </FormField>
              <FormField label={t('uart.parity')}>
                <SelectField
                  value={formPort.parity}
                  onChange={(event) => setFormPort((current) => ({ ...current, parity: event.target.value as UartParity }))}
                  options={parityOptions}
                />
              </FormField>
              <FormField label={t('uart.stopBits')}>
                <SelectField
                  value={formPort.stopBits}
                  onChange={(event) =>
                    setFormPort((current) => ({ ...current, stopBits: event.target.value as UartStopBits }))
                  }
                  options={stopBitsOptions}
                />
              </FormField>
              <FormField label={t('uart.flowControl')}>
                <SelectField
                  value={formPort.flowControl}
                  onChange={(event) =>
                    setFormPort((current) => ({ ...current, flowControl: event.target.value as UartFlowControl }))
                  }
                  options={flowControlOptions}
                />
              </FormField>
              <FormField label={t('uart.encoding')}>
                <SelectField
                  value={formPort.encoding ?? 'auto'}
                  onChange={(event) =>
                    setFormPort((current) => ({
                      ...current,
                      encoding: event.target.value === 'auto' ? undefined : event.target.value,
                    }))
                  }
                  options={terminalEncodingOptionsWithAuto(t('uart.encodingAuto'))}
                />
              </FormField>
            </>
          )}
        </div>
        <div className="flex flex-wrap items-center gap-3">
          {isEditing && (
            <button
              type="button"
              onClick={resetForm}
              className="inline-flex items-center gap-2 px-4 py-1.5 bg-golden-cream text-warm-black rounded-standard font-medium text-xs border border-golden-sand hover:bg-pale-sand transition-colors duration-vt-base"
            >
              <X className="w-4 h-4" />
              {t('common.cancel')}
            </button>
          )}
          <button
            type="button"
            onClick={handleSubmitPort}
            className="inline-flex items-center gap-2 px-4 py-1.5 bg-amber-gold text-text-inverse rounded-standard font-medium text-xs hover:opacity-90 transition-opacity border border-amber-gold"
          >
            <Plus className="w-4 h-4" />
            {isEditing ? t('uart.updatePort') : t('uart.addPort')}
          </button>
        </div>
      </div>

      <ConfirmDialog
        open={removeTarget !== null}
        title={t('uart.removeTitle')}
        message={t('uart.removeMessage')}
        confirmLabel={t('common.remove')}
        variant="danger"
        onConfirm={handleRemovePort}
        onCancel={() => setRemoveTarget(null)}
      />

      {(saving || saveError) && (
        <div className={`text-xs ${saveError ? 'text-vt-error' : 'text-warm-khaki'}`}>
          {saveError ?? t('common.saving')}
        </div>
      )}
    </div>
  );
}
