import { useCallback, useEffect, useMemo, useState } from 'react';
import { Loader2, AlertCircle, AlertTriangle, ChevronDown, ChevronUp, ClipboardPaste } from 'lucide-react';
import { QRCodeSVG } from 'qrcode.react';
import { ConfirmDialog } from '@/components/shared/ConfirmDialog';
import { getServerConfig, updateServerConfig, openFullDiskAccessSettings } from '@/api/server';
import { getRootCaPem } from '@/api/certificates';
import { useConfigSection } from '@/hooks/use-config-section';
import { useToast } from '@/components/shared/Toast';
import { FormField } from '@/components/shared/FormField';
import { NumberInput } from '@/components/shared/NumberInput';
import { SecretField } from '@/components/shared/SecretField';
import { SelectField } from '@/components/shared/SelectField';
import { ToggleField } from '@/components/shared/ToggleField';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { useTranslation } from '@/i18n';
import { copyToClipboard } from '@/utils/clipboard';
import type { IpFamily, ServerConfig, ServerNetworkInterface } from '@/types';

const QR_SECRET_PREFIX = 'vtqr1:';
const SHARE_PREFIX = 'vtshare1:';
// XTEA key shared with the desktop client (vterm share_payload.rs).
// The webui derives 4 big-endian words from this ASCII string; the missing
// 16th byte reads as 0 there, hence the trailing NUL in the rust constant.
const SHARE_XTEA_KEY = 'VibeTermQrKeyV1';
const XTEA_BLOCK_SIZE = 8;
const XTEA_DELTA = 0x9e3779b9;
const XTEA_ROUNDS = 32;

export function ServerSection() {
  const { addToast } = useToast();
  const { t } = useTranslation();
  const [selectedInterfaceId, setSelectedInterfaceId] = useState('');
  const [ipFamily, setIpFamily] = useState<IpFamily>('ipv4');
  const [connectionType, setConnectionType] = useState<'ip' | 'domain'>('ip');
  const [showGenerateConfirm, setShowGenerateConfirm] = useState(false);
  const [qrExpanded, setQrExpanded] = useState(false);
  const [relayStrictExpanded, setRelayStrictExpanded] = useState(false);
  const [caPem, setCaPem] = useState<string | null>(null);
  // Relay connection settings stay in local drafts until the apply button is
  // clicked: auto-saving them per keystroke would reconnect the upstream relay
  // with half-typed values.
  const [relayAddressDraft, setRelayAddressDraft] = useState<string | null>(null);
  const [relaySecretKeyDraft, setRelaySecretKeyDraft] = useState<string | null>(null);
  const [relayStrictTlsDraft, setRelayStrictTlsDraft] = useState<boolean | null>(null);
  const [relayCaPemDraft, setRelayCaPemDraft] = useState<string | null>(null);
  // The raw clipboard text while it carries a `vtshare1:` share string (the
  // paste icon next to the relay address shows only while this is set).
  const [clipboardShare, setClipboardShare] = useState<string | null>(null);
  const fetchFn = useCallback(() => getServerConfig(), []);
  const saveFn = useCallback((data: ServerConfig) => {
    const validationError = secretKeyValidationError(data, t);
    if (validationError) {
      return Promise.reject(new Error(validationError));
    }
    return updateServerConfig(data);
  }, [t]);

  const { data, loading, saving, loadError, saveError, updateField, updateData, save } =
    useConfigSection<ServerConfig>(fetchFn, saveFn);
  const networkInterfaces = data?.networkInterfaces ?? [];
  const ddnsDomain = data?.ddnsDomain ?? '';
  const serverName = data?.name ?? '';
  const serverPort = data?.port ?? 0;
  const secretKey = data?.secretKey ?? '';
  const encryptedSecretKey = useMemo(() => encryptQrSecretKey(secretKey), [secretKey]);

  useEffect(() => {
    let active = true;
    getRootCaPem()
      .then((pem) => {
        if (active) setCaPem(pem);
      })
      .catch(() => {
        if (active) setCaPem('');
      });
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    if (networkInterfaces.length === 0) {
      if (selectedInterfaceId) {
        setSelectedInterfaceId('');
      }
      return;
    }

    if (!networkInterfaces.some((networkInterface) => networkInterface.id === selectedInterfaceId)) {
      setSelectedInterfaceId(networkInterfaces[0].id);
    }
  }, [networkInterfaces, selectedInterfaceId]);

  const selectedInterface = useMemo(
    () =>
      networkInterfaces.find((networkInterface) => networkInterface.id === selectedInterfaceId) ??
      networkInterfaces[0],
    [networkInterfaces, selectedInterfaceId]
  );

  useEffect(() => {
    if (!selectedInterface) {
      return;
    }

    const availableFamilies = getAvailableFamilies(selectedInterface);
    if (availableFamilies.length > 0 && !availableFamilies.includes(ipFamily)) {
      setIpFamily(availableFamilies[0]);
    }
  }, [selectedInterface, ipFamily]);

  const selectedIpAddress = useMemo(
    () => (selectedInterface ? getAddressForFamily(selectedInterface, ipFamily) : ''),
    [selectedInterface, ipFamily]
  );

  const connectionAddress = useMemo(() => {
    if (connectionType === 'domain' && ddnsDomain) {
      return ddnsDomain;
    }
    return selectedIpAddress;
  }, [connectionType, ddnsDomain, selectedIpAddress]);

  const connectionUrl = useMemo(() => {
    if (!connectionAddress || !serverPort) return '';
    // IPv6 literal addresses must be wrapped in brackets for URLs (RFC 3986)
    const addr = ipFamily === 'ipv6' && connectionType === 'ip' ? `[${connectionAddress}]` : connectionAddress;
    return `wss://${addr}:${serverPort}`;
  }, [connectionAddress, connectionType, ipFamily, serverPort]);

  const qrPayload = useMemo(() => {
    if (!data || !connectionAddress) {
      return '';
    }

    return JSON.stringify({
      type: 'vibe-terminal-server',
      domain: connectionType === 'domain' ? ddnsDomain : '',
      ip: connectionType === 'ip' && ipFamily === 'ipv6' ? `[${selectedIpAddress}]` : connectionType === 'ip' ? selectedIpAddress : '',
      port: serverPort,
      secret_key: encryptedSecretKey,
      name: serverName,
    });
  }, [connectionAddress, connectionType, ddnsDomain, encryptedSecretKey, selectedIpAddress, serverName, serverPort]);

  // Encrypted share string copied by clicking the QR code. The desktop
  // client detects this prefix on its add-server form and decrypts it to
  // auto-fill the fields (crates/vterm .../share_payload.rs).
  const shareString = useMemo(() => {
    if (!data || !connectionAddress || !serverPort || caPem === null) {
      return '';
    }

    return encryptWithXteaCbc(
      JSON.stringify({
        type: 'vibe-terminal-server',
        name: serverName,
        // Raw address literal (no IPv6 brackets — the client re-wraps).
        host: connectionAddress,
        port: serverPort,
        secret_key: secretKey,
        ...(caPem ? { ca_pem: caPem } : {}),
      }),
      SHARE_PREFIX,
    );
  }, [caPem, connectionAddress, data, secretKey, serverName, serverPort]);

  const handleCopyShareString = async () => {
    if (!shareString) return;
    const copied = await copyToClipboard(shareString);
    addToast(copied ? 'success' : 'error', t(copied ? 'server.qrCopied' : 'server.qrCopyFailed'));
  };

  // Best-effort clipboard probe: shows the relay paste icon while the
  // clipboard carries a share string. Re-checked on window focus and when
  // the relay address field is focused; silently hidden when the clipboard
  // is unreadable (plain-HTTP contexts have no navigator.clipboard).
  const probeClipboardShare = useCallback(async () => {
    try {
      const text = await navigator.clipboard.readText();
      setClipboardShare(text.trim().startsWith(SHARE_PREFIX) ? text : null);
    } catch {
      setClipboardShare(null);
    }
  }, []);

  useEffect(() => {
    void probeClipboardShare();
    window.addEventListener('focus', probeClipboardShare);
    return () => window.removeEventListener('focus', probeClipboardShare);
  }, [probeClipboardShare]);

  const handlePasteRelayShare = async () => {
    let text: string | null = null;
    try {
      text = await navigator.clipboard.readText();
    } catch {
      // fall through — reported as invalid below
    }
    const share = text ? decryptShareString(text) : null;
    if (!share) {
      addToast('error', t('server.relayPasteShareFailed'));
      return;
    }
    // Fill the drafts (not the saved config): the user reviews and clicks
    // Apply, which is also what reconnects the upstream relay.
    setRelayAddressDraft(share.address);
    setRelaySecretKeyDraft(share.secretKey);
    setRelayCaPemDraft(share.caPem);
    if (!isValidCaPem(share.caPem)) {
      setRelayStrictTlsDraft(false);
    }
    addToast('success', t('server.relayPasteShareDone'));
  };

  const handleConfirmGenerateKey = () => {
    updateField('secretKey', generateSecretKey());
    addToast('success', t('server.keyGenerated'));
    setShowGenerateConfirm(false);
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
        <span className="font-light">{loadError ?? t('server.loadError')}</span>
      </div>
    );
  }

  const inputClassName =
    'w-full px-2 py-1.5 bg-bg-inset border border-golden-sand rounded-standard text-warm-black text-sm placeholder:text-warm-khaki focus:outline-none focus:border-amber-gold';

  const relayAddressValue = relayAddressDraft ?? data.relayAddress;
  const relaySecretKeyValue = relaySecretKeyDraft ?? data.relaySecretKey;
  const relayStrictTlsValue = relayStrictTlsDraft ?? data.relayStrictTls ?? false;
  const relayCaPemValue = relayCaPemDraft ?? data.relayCaPem ?? '';
  const relayCaPemIsValid = isValidCaPem(relayCaPemValue);
  const relayAddressDirty = relayAddressDraft !== null && relayAddressDraft.trim() !== data.relayAddress;
  const relaySecretKeyDirty = relaySecretKeyDraft !== null && relaySecretKeyDraft.trim() !== data.relaySecretKey;
  const relayStrictTlsDirty = relayStrictTlsDraft !== null && relayStrictTlsDraft !== data.relayStrictTls;
  const relayCaPemDirty = relayCaPemDraft !== null && relayCaPemDraft.trim() !== (data.relayCaPem ?? '');

  // Applies the relay drafts together because they describe one upstream
  // connection, and persisting the config reconnects it (see relay.rs).
  const handleApplyRelay = async () => {
    const next: ServerConfig = {
      ...data,
      relayAddress: relayAddressValue.trim(),
      relaySecretKey: relaySecretKeyValue.trim(),
      relayStrictTls: relayStrictTlsValue,
      relayCaPem: relayCaPemValue.trim(),
    };
    const validationError = secretKeyValidationError(next, t);
    if (validationError) {
      addToast('error', validationError);
      return;
    }
    setRelayAddressDraft(null);
    setRelaySecretKeyDraft(null);
    setRelayStrictTlsDraft(null);
    setRelayCaPemDraft(null);
    updateData({
      relayAddress: next.relayAddress,
      relaySecretKey: next.relaySecretKey,
      relayStrictTls: next.relayStrictTls,
      relayCaPem: next.relayCaPem,
    });
    const saved = await save(next);
    addToast(saved ? 'success' : 'error', t(saved ? 'server.relayApplied' : 'server.relayApplyFailed'));
  };

  // Opens the macOS Full Disk Access pane on the server host (the button is
  // only rendered for macOS servers).
  const handleOpenFullDiskAccess = async () => {
    try {
      await openFullDiskAccessSettings();
      addToast('success', t('server.fullDiskAccessOpened'));
    } catch {
      addToast('error', t('server.fullDiskAccessFailed'));
    }
  };

  const applyButton = (visible: boolean) =>
    visible ? (
      <button
        type="button"
        onClick={handleApplyRelay}
        disabled={saving}
        className="shrink-0 inline-flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-standard text-xs font-semibold bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {t('server.relayApply')}
      </button>
    ) : null;

  return (
    <div className="space-y-6">
      <SectionHeader title={t('server.title')} description={t('server.description')} />

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <FormField label={t('server.port')} description={t('server.portDesc')}>
          <NumberInput value={data.port} onChange={(v) => updateField('port', v)} min={1} max={65535} />
        </FormField>

        <FormField label={t('server.configPort')} description={t('server.configPortDesc')}>
          <NumberInput value={data.configPort} onChange={(v) => updateField('configPort', v)} min={1} max={65535} />
        </FormField>

        <FormField label={t('server.name')} description={t('server.nameDesc')}>
          <input
            type="text"
            value={data.name}
            onChange={(e) => updateField('name', e.target.value)}
            className={inputClassName}
            placeholder="Vibe Terminal"
          />
        </FormField>

        <FormField label={t('server.secretKey')} description={t('server.secretKeyDesc')}>
          {/* Dice (not the field) triggers the confirm dialog: regenerating
              invalidates every paired client, so it stays a guarded action
              even as an inline icon. */}
          <SecretField
            value={data.secretKey}
            onChange={(e) => updateField('secretKey', e.target.value)}
            placeholder={t('server.secretKeyPlaceholder')}
            required
            minLength={12}
            maxLength={16}
            copyText={data.secretKey}
            onGenerate={() => setShowGenerateConfirm(true)}
            generateTitle={t('server.generateKey')}
          />
        </FormField>

        <FormField label={t('server.relayAddress')} description={t('server.relayAddressDesc')}>
          <div className="flex items-center gap-2">
            <div className="flex-1">
              <input
                type="text"
                value={relayAddressValue}
                onChange={(e) => setRelayAddressDraft(e.target.value)}
                onFocus={() => void probeClipboardShare()}
                className={inputClassName}
                placeholder="wss://relay.example.com:5153"
              />
            </div>
            {clipboardShare && (
              <button
                type="button"
                onClick={handlePasteRelayShare}
                title={t('server.relayPasteShare')}
                aria-label={t('server.relayPasteShare')}
                className="shrink-0 p-1.5 rounded-standard text-deep-olive hover:text-warm-black hover:bg-pale-sand border border-golden-sand transition-colors duration-vt-base"
              >
                <ClipboardPaste className="w-4 h-4" />
              </button>
            )}
            {applyButton(relayAddressDirty)}
          </div>
        </FormField>

        <FormField label={t('server.relaySecretKey')} description={t('server.relaySecretKeyDesc')}>
          {/* Copy + eye only: the relay key is issued by the relay service,
              so a dice that writes a random string here would be wrong. */}
          <div className="flex items-center gap-2">
            <div className="flex-1">
              <SecretField
                value={relaySecretKeyValue}
                onChange={(e) => setRelaySecretKeyDraft(e.target.value)}
                placeholder={t('server.relaySecretKeyPlaceholder')}
                required={relayAddressValue.trim().length > 0}
                minLength={12}
                maxLength={16}
                copyText={data.relaySecretKey}
              />
            </div>
            {applyButton(relaySecretKeyDirty)}
          </div>
        </FormField>

        <div className="space-y-3">
          <button
            type="button"
            onClick={() => setRelayStrictExpanded((expanded) => !expanded)}
            aria-expanded={relayStrictExpanded}
            className="w-full flex items-center justify-between gap-3 px-3 py-2 bg-bg-inset border border-golden-sand rounded-standard text-sm text-warm-black hover:border-amber-gold focus:outline-none focus:border-amber-gold transition-colors duration-vt-base"
          >
            <span className="font-medium">{t('server.relayStrictTls')}</span>
            <span className="inline-flex items-center gap-2 text-xs font-semibold">
              <span className={relayStrictTlsValue ? 'text-deep-olive' : 'text-warm-khaki'}>
                {relayStrictTlsValue ? 'On' : 'Off'}
              </span>
              {relayStrictExpanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
            </span>
          </button>

          {relayStrictExpanded && (
            <div className="space-y-3 rounded-standard border border-golden-sand bg-bg-inset/40 p-3">
              <p className="text-xs font-normal text-warm-khaki">{t('server.relayStrictTlsDesc')}</p>
              <FormField
                label={t('server.relayCaPem')}
                description={t('server.relayCaPemDesc')}
                error={relayCaPemValue.trim().length > 0 && !relayCaPemIsValid ? t('server.relayCaPemInvalid') : undefined}
              >
                <textarea
                  value={relayCaPemValue}
                  onChange={(e) => {
                    const value = e.target.value;
                    setRelayCaPemDraft(value);
                    if (!isValidCaPem(value)) {
                      setRelayStrictTlsDraft(false);
                    }
                  }}
                  rows={5}
                  spellCheck={false}
                  placeholder="-----BEGIN CERTIFICATE-----"
                  className={`${inputClassName} resize-y font-mono text-xs`}
                />
              </FormField>
              <div className="flex items-center gap-2">
                <div className="inline-flex rounded-standard border border-golden-sand p-0.5" aria-label={t('server.relayStrictTls')}>
                  <button
                    type="button"
                    onClick={() => setRelayStrictTlsDraft(false)}
                    aria-pressed={!relayStrictTlsValue}
                    className={`px-3 py-1 rounded-standard text-xs font-semibold transition-colors duration-vt-base ${
                      !relayStrictTlsValue ? 'bg-warm-black text-golden-cream' : 'text-warm-black hover:bg-pale-sand'
                    }`}
                  >
                    Off
                  </button>
                  <button
                    type="button"
                    onClick={() => setRelayStrictTlsDraft(true)}
                    disabled={!relayCaPemIsValid}
                    title={!relayCaPemIsValid ? t('server.relayCaPemInvalid') : undefined}
                    aria-pressed={relayStrictTlsValue}
                    className={`px-3 py-1 rounded-standard text-xs font-semibold transition-colors duration-vt-base disabled:opacity-40 disabled:cursor-not-allowed ${
                      relayStrictTlsValue ? 'bg-amber-gold text-text-inverse' : 'text-warm-black hover:bg-pale-sand'
                    }`}
                  >
                    On
                  </button>
                </div>
                {applyButton(relayStrictTlsDirty || relayCaPemDirty)}
              </div>
            </div>
          )}
        </div>

        <FormField label={t('server.logToFile')} description={t('server.logToFileDesc')}>
          <ToggleField enabled={data.logToFile} onChange={(v) => updateField('logToFile', v)} />
        </FormField>

        {data.autostartSupported !== false && (
          <FormField label={t('server.autoStart')} description={t('server.autoStartDesc')}>
            <ToggleField enabled={data.autostart} onChange={(v) => updateField('autostart', v)} />
          </FormField>
        )}

        {data.autostartSupported !== false && data.platform === 'linux' && (
          <p className="text-xs font-normal text-warm-khaki">{t('server.autoStartLinuxNote')}</p>
        )}

        {data.platform === 'macos' && (
          <div className="flex items-start gap-2 rounded-standard border border-vt-warning bg-vt-warning/25 px-3 py-2">
            <AlertTriangle className="w-3.5 h-3.5 shrink-0 mt-0.5 text-warm-black" />
            <div className="space-y-1.5">
              <p className="text-xs font-medium text-warm-black">{t('server.fullDiskAccess')}</p>
              <p className="text-xs text-warm-black leading-relaxed">{t('server.fullDiskAccessDesc')}</p>
              <button
                type="button"
                onClick={() => void handleOpenFullDiskAccess()}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-standard text-xs font-semibold bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90 transition-opacity duration-vt-base"
              >
                {t('server.fullDiskAccessOpen')}
              </button>
            </div>
          </div>
        )}
      </div>

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <SectionHeader title={t('server.qrCode')} description={t('server.qrCodeDesc')}>
          <button
            type="button"
            onClick={() => setQrExpanded((v) => !v)}
            className="p-1.5 rounded-standard text-deep-olive hover:text-warm-black hover:bg-pale-sand transition-colors duration-vt-base"
            aria-label={qrExpanded ? 'Collapse QR code section' : 'Expand QR code section'}
          >
            {qrExpanded ? <ChevronUp className="w-5 h-5" /> : <ChevronDown className="w-5 h-5" />}
          </button>
        </SectionHeader>

        {qrExpanded && (
          <>
            <FormField label={t('server.networkInterface')}>
              <SelectField
                value={selectedInterface?.id ?? ''}
                onChange={(e) => setSelectedInterfaceId(e.target.value)}
                disabled={networkInterfaces.length === 0}
                options={
                  networkInterfaces.length === 0
                    ? [{ value: '', label: t('server.noInterfaces') }]
                    : networkInterfaces.map((networkInterface) => ({
                        value: networkInterface.id,
                        label: networkInterface.name,
                      }))
                }
              />
            </FormField>

            <FormField label={t('server.ipFamily')}>
              <SelectField
                value={ipFamily}
                onChange={(e) => setIpFamily(e.target.value as IpFamily)}
                disabled={!selectedInterface}
                options={[
                  {
                    value: 'ipv4',
                    label: 'IPv4',
                    disabled: !selectedInterface || selectedInterface.ipv4Addresses.length === 0,
                  },
                  {
                    value: 'ipv6',
                    label: 'IPv6',
                    disabled: !selectedInterface || selectedInterface.ipv6Addresses.length === 0,
                  },
                ]}
              />
            </FormField>

            {ddnsDomain && (
              <FormField label={t('server.connectionType')}>
                <SelectField
                  value={connectionType}
                  onChange={(e) => setConnectionType(e.target.value as 'ip' | 'domain')}
                  options={[
                    { value: 'ip', label: t('server.connectionIp') },
                    { value: 'domain', label: t('server.connectionDomain') },
                  ]}
                />
              </FormField>
            )}

            <div className="grid grid-cols-1 lg:grid-cols-[240px,1fr] gap-6 items-start">
              <div className="flex flex-col items-center gap-3">
                <div className="bg-golden-cream border border-golden-sand rounded-comfortable p-4 min-h-[240px] flex items-center justify-center">
                  {qrPayload ? (
                    <button
                      type="button"
                      onClick={handleCopyShareString}
                      disabled={!shareString}
                      title={t('server.qrShareHint')}
                      className="block cursor-pointer disabled:cursor-default rounded-standard transition-opacity duration-vt-base hover:opacity-80 disabled:hover:opacity-100 focus:outline-none"
                    >
                      <QRCodeSVG
                        value={qrPayload}
                        size={208}
                        bgColor="transparent"
                        fgColor="currentColor"
                        className="text-warm-black"
                      />
                    </button>
                  ) : (
                    <div className="text-xs font-light text-warm-khaki text-center">
                      {networkInterfaces.length === 0 ? t('server.noInterfaces') : t('server.noAddressForFamily')}
                    </div>
                  )}
                </div>
                <p className="text-[11px] leading-vt-sub text-warm-khaki text-center max-w-[240px]">
                  {t('server.qrShareHint')}
                </p>
              </div>

              <div className="space-y-4">
                <div>
                  <div className="text-xs font-medium text-warm-khaki mb-2">{t('server.qrPreview')}</div>
                  <div className="bg-golden-cream border border-golden-sand rounded-comfortable p-4 space-y-3">
                    <div>
                      <div className="text-[11px] font-medium text-warm-khaki mb-1">{t('server.connectionUrl')}</div>
                      <div className="text-xs text-warm-black font-mono break-all">
                        {connectionUrl || t('server.noAddressForFamily')}
                      </div>
                    </div>
                    <div>
                      <div className="text-[11px] font-medium text-warm-khaki mb-1">
                        {connectionType === 'domain' ? t('server.domain') : t('server.ipAddress')}
                      </div>
                      <div className="text-xs text-warm-black font-mono break-all">
                        {connectionAddress || t('server.noAddressForFamily')}
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </>
        )}
      </div>

      <ConfirmDialog
        open={showGenerateConfirm}
        title={t('server.generateKeyTitle')}
        message={t('server.generateKeyMessage')}
        onConfirm={handleConfirmGenerateKey}
        onCancel={() => setShowGenerateConfirm(false)}
      />

      {(saving || saveError) && (
        <div className={`text-xs ${saveError ? 'text-vt-error' : 'text-warm-khaki'}`}>
          {saveError ?? t('common.saving')}
        </div>
      )}
    </div>
  );
}

function getAvailableFamilies(networkInterface: ServerNetworkInterface): IpFamily[] {
  return [
    ...(networkInterface.ipv4Addresses.length > 0 ? (['ipv4'] as const) : []),
    ...(networkInterface.ipv6Addresses.length > 0 ? (['ipv6'] as const) : []),
  ];
}

function getAddressForFamily(networkInterface: ServerNetworkInterface, ipFamily: IpFamily): string {
  return ipFamily === 'ipv6'
    ? networkInterface.ipv6Addresses[0] ?? ''
    : networkInterface.ipv4Addresses[0] ?? '';
}

function generateSecretKey(): string {
  const alphabet = 'ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789';
  const length = 12 + (crypto.getRandomValues(new Uint32Array(1))[0] % 5);
  const randomValues = crypto.getRandomValues(new Uint32Array(length));

  return Array.from(randomValues, (value) => alphabet[value % alphabet.length]).join('');
}

function encryptQrSecretKey(secretKey: string): string {
  return encryptWithXteaCbc(secretKey, QR_SECRET_PREFIX);
}

function isValidCaPem(pem: string): boolean {
  const pattern = /-----BEGIN CERTIFICATE-----\s*([A-Za-z0-9+/=\s]+?)-----END CERTIFICATE-----/g;
  let previousEnd = 0;
  let certificateCount = 0;

  for (const match of pem.matchAll(pattern)) {
    const index = match.index ?? 0;
    if (pem.slice(previousEnd, index).trim() !== '' || !hasDerCertificateEnvelope(match[1] ?? '')) {
      return false;
    }
    previousEnd = index + match[0].length;
    certificateCount += 1;
  }

  return certificateCount > 0 && pem.slice(previousEnd).trim() === '';
}

function hasDerCertificateEnvelope(base64: string): boolean {
  const compact = base64.replace(/\s/g, '');
  if (compact.length === 0 || compact.length % 4 !== 0 || !/^[A-Za-z0-9+/]+={0,2}$/.test(compact)) {
    return false;
  }

  try {
    const der = atob(compact);
    if (der.length < 4 || der.charCodeAt(0) !== 0x30) {
      return false;
    }

    let offset = 1;
    const firstLengthByte = der.charCodeAt(offset);
    offset += 1;
    let contentLength = 0;
    if ((firstLengthByte & 0x80) === 0) {
      contentLength = firstLengthByte;
    } else {
      const lengthByteCount = firstLengthByte & 0x7f;
      if (lengthByteCount === 0 || lengthByteCount > 4 || offset + lengthByteCount >= der.length) {
        return false;
      }
      for (let index = 0; index < lengthByteCount; index += 1) {
        contentLength = contentLength * 256 + der.charCodeAt(offset);
        offset += 1;
      }
    }
    return offset + contentLength === der.length;
  } catch {
    return false;
  }
}

function secretKeyValidationError(
  config: ServerConfig,
  t: (key: string) => string,
): string | null {
  const secretKey = config.secretKey.trim();
  if (!secretKey) {
    return t('server.secretKeyRequired');
  }
  if (!isProtectedSecretKey(secretKey) && (secretKey.length < 12 || secretKey.length > 16)) {
    return t('server.secretKeyLength');
  }

  const relaySecretKey = config.relaySecretKey.trim();
  if (config.relayAddress.trim() && !relaySecretKey) {
    return t('server.relaySecretKeyRequired');
  }
  if (relaySecretKey && !isProtectedSecretKey(relaySecretKey) && (relaySecretKey.length < 12 || relaySecretKey.length > 16)) {
    return t('server.relaySecretKeyLength');
  }
  return null;
}

function isProtectedSecretKey(value: string): boolean {
  return value.startsWith('sha256:') || value.startsWith('enc-v');
}

/** Server info decoded from a `vtshare1:` string (relay auto-fill). */
interface SharedServer {
  /** `wss://host:port` with IPv6 literals bracketed. */
  address: string;
  secretKey: string;
  caPem: string;
}

/**
 * Parse and decrypt a `vtshare1:<iv hex>:<ciphertext hex>` share string —
 * the mirror of the desktop client's `parse_share_string`
 * (crates/vterm .../share_payload.rs). Returns `null` for any other
 * input.
 */
function decryptShareString(text: string): SharedServer | null {
  const rest = text.trim();
  if (!rest.startsWith(SHARE_PREFIX)) {
    return null;
  }
  const [ivHex, cipherHex] = rest.slice(SHARE_PREFIX.length).split(':');
  if (!ivHex || !cipherHex || ivHex.length !== XTEA_BLOCK_SIZE * 2 || cipherHex.length % (XTEA_BLOCK_SIZE * 2) !== 0) {
    return null;
  }
  const iv = decodeHex(ivHex);
  const cipher = decodeHex(cipherHex);
  if (!iv || !cipher || cipher.length === 0) {
    return null;
  }

  const keyWords = qrSecretKeyWords();
  const plain = new Uint8Array(cipher.length);
  let previousBlock = iv;
  for (let offset = 0; offset < cipher.length; offset += XTEA_BLOCK_SIZE) {
    const [pl, pr] = decryptXteaBlock(bytesToWord(cipher, offset), bytesToWord(cipher, offset + 4), keyWords);
    wordToBytes(pl, plain, offset);
    wordToBytes(pr, plain, offset + XTEA_BLOCK_SIZE / 2);
    for (let index = 0; index < XTEA_BLOCK_SIZE; index += 1) {
      plain[offset + index] ^= previousBlock[index];
    }
    previousBlock = cipher.slice(offset, offset + XTEA_BLOCK_SIZE);
  }

  // The webui pads with 1..=8 bytes of the pad length (8 when aligned).
  const padding = plain[plain.length - 1] ?? 0;
  if (padding < 1 || padding > XTEA_BLOCK_SIZE || padding > plain.length) {
    return null;
  }

  try {
    const payload = JSON.parse(new TextDecoder().decode(plain.subarray(0, plain.length - padding))) as {
      host?: string;
      port?: number;
      secret_key?: string;
      ca_pem?: string;
    };
    if (!payload.host || typeof payload.port !== 'number' || payload.port <= 0) {
      return null;
    }
    const host = payload.host.includes(':') ? `[${payload.host}]` : payload.host;
    return {
      address: `wss://${host}:${payload.port}`,
      secretKey: payload.secret_key ?? '',
      caPem: payload.ca_pem ?? '',
    };
  } catch {
    return null;
  }
}

function decryptXteaBlock(left: number, right: number, keyWords: Uint32Array): [number, number] {
  let v0 = left >>> 0;
  let v1 = right >>> 0;
  let sum = (XTEA_DELTA * XTEA_ROUNDS) >>> 0;

  for (let round = 0; round < XTEA_ROUNDS; round += 1) {
    v1 =
      (v1 - ((((v0 << 4) ^ (v0 >>> 5)) + v0) ^ (sum + keyWords[(sum >>> 11) & 3]))) >>> 0;
    sum = (sum - XTEA_DELTA) >>> 0;
    v0 =
      (v0 - ((((v1 << 4) ^ (v1 >>> 5)) + v1) ^ (sum + keyWords[sum & 3]))) >>> 0;
  }

  return [v0, v1];
}

function decodeHex(hex: string): Uint8Array | null {
  if (hex.length % 2 !== 0) {
    return null;
  }
  const bytes = new Uint8Array(hex.length / 2);
  for (let index = 0; index < bytes.length; index += 1) {
    const value = Number.parseInt(hex.slice(index * 2, index * 2 + 2), 16);
    if (Number.isNaN(value)) {
      return null;
    }
    bytes[index] = value;
  }
  return bytes;
}

/**
 * XTEA-CBC encrypt `plain` and format as `<prefix><iv hex>:<ciphertext hex>`.
 * Shared by the QR secret key (`vtqr1:`) and the client share string
 * (`vtshare1:`) — the desktop client implements the matching decrypt.
 */
function encryptWithXteaCbc(plain: string, prefix: string): string {
  if (!plain) {
    return '';
  }

  const plainBytes = new TextEncoder().encode(plain);
  const paddedBytes = padQrSecretBytes(plainBytes);
  const encryptedBytes = new Uint8Array(paddedBytes.length);
  const iv = crypto.getRandomValues(new Uint8Array(XTEA_BLOCK_SIZE));
  const keyWords = qrSecretKeyWords();
  let previousBlock = iv;

  for (let offset = 0; offset < paddedBytes.length; offset += XTEA_BLOCK_SIZE) {
    const block = paddedBytes.slice(offset, offset + XTEA_BLOCK_SIZE);
    for (let index = 0; index < XTEA_BLOCK_SIZE; index += 1) {
      block[index] ^= previousBlock[index];
    }

    const [left, right] = encryptXteaBlock(bytesToWord(block, 0), bytesToWord(block, 4), keyWords);
    wordToBytes(left, encryptedBytes, offset);
    wordToBytes(right, encryptedBytes, offset + 4);
    previousBlock = encryptedBytes.slice(offset, offset + XTEA_BLOCK_SIZE);
  }

  return `${prefix}${encodeHex(iv)}:${encodeHex(encryptedBytes)}`;
}

function padQrSecretBytes(bytes: Uint8Array): Uint8Array {
  const remainder = bytes.length % XTEA_BLOCK_SIZE;
  const paddingLength = remainder === 0 ? XTEA_BLOCK_SIZE : XTEA_BLOCK_SIZE - remainder;
  const paddedBytes = new Uint8Array(bytes.length + paddingLength);
  paddedBytes.set(bytes);
  paddedBytes.fill(paddingLength, bytes.length);
  return paddedBytes;
}

function qrSecretKeyWords(): Uint32Array {
  const keyBytes = new TextEncoder().encode(SHARE_XTEA_KEY);
  return new Uint32Array([
    bytesToWord(keyBytes, 0),
    bytesToWord(keyBytes, 4),
    bytesToWord(keyBytes, 8),
    bytesToWord(keyBytes, 12),
  ]);
}

function encryptXteaBlock(left: number, right: number, keyWords: Uint32Array): [number, number] {
  let v0 = left >>> 0;
  let v1 = right >>> 0;
  let sum = 0;

  for (let round = 0; round < XTEA_ROUNDS; round += 1) {
    v0 =
      (v0 +
        ((((v1 << 4) ^ (v1 >>> 5)) + v1) ^ (sum + keyWords[sum & 3]))) >>>
      0;
    sum = (sum + XTEA_DELTA) >>> 0;
    v1 =
      (v1 +
        ((((v0 << 4) ^ (v0 >>> 5)) + v0) ^ (sum + keyWords[(sum >>> 11) & 3]))) >>>
      0;
  }

  return [v0, v1];
}

function bytesToWord(bytes: Uint8Array, offset: number): number {
  return (
    ((bytes[offset] ?? 0) << 24) |
    ((bytes[offset + 1] ?? 0) << 16) |
    ((bytes[offset + 2] ?? 0) << 8) |
    (bytes[offset + 3] ?? 0)
  ) >>> 0;
}

function wordToBytes(word: number, bytes: Uint8Array, offset: number): void {
  bytes[offset] = (word >>> 24) & 0xff;
  bytes[offset + 1] = (word >>> 16) & 0xff;
  bytes[offset + 2] = (word >>> 8) & 0xff;
  bytes[offset + 3] = word & 0xff;
}

function encodeHex(bytes: Uint8Array): string {
  return Array.from(bytes, (value) => value.toString(16).padStart(2, '0')).join('');
}
