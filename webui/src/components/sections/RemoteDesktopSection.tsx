import { useCallback, useEffect, useState } from 'react';
import { Loader2, AlertCircle, ShieldCheck, ShieldAlert, ShieldQuestion, MonitorOff, Copy } from 'lucide-react';
import {
  getRemoteDesktopConfig,
  updateRemoteDesktopConfig,
  requestRemoteDesktopPermission,
} from '@/api/remote-desktop';
import {
  fetchScreenPaths,
  saveScreenPaths,
  fetchScreenEncoder,
  saveScreenEncoder,
  fetchScreenService,
  installScreenService,
  removeScreenService,
  fetchScreenInputStatus,
  resetScreenConsent,
} from '@/api/screen';
import type { ScreenInputStatusResponse, ScreenServiceResponse } from '@/types';
import { useConfigSection } from '@/hooks/use-config-section';
import { useToast } from '@/components/shared/Toast';
import { ToggleField } from '@/components/shared/ToggleField';
import { SelectField } from '@/components/shared/SelectField';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { SaveResetButtons } from '@/components/shared/SaveResetButtons';
import { useTranslation } from '@/i18n';
import type { RemoteDesktopConfig, ScreenEncoderResponse, ScreenPathInfo, ScreenPathsResponse } from '@/types';

export function RemoteDesktopSection() {
  const { addToast } = useToast();
  const { t } = useTranslation();
  // The grant action can change the OS permission status out-of-band, so the
  // displayed status is held locally (seeded from the server) rather than only
  // from the saved config.
  const [permissionGranted, setPermissionGranted] = useState<boolean | null>(null);
  const [requesting, setRequesting] = useState(false);

  const fetchFn = useCallback(async () => {
    const config = await getRemoteDesktopConfig();
    setPermissionGranted(config.permissionGranted);
    return config;
  }, []);
  const saveFn = useCallback(
    (data: RemoteDesktopConfig) => updateRemoteDesktopConfig(data),
    [],
  );

  const { data, loading, saving, loadError, saveError, isDirty, updateField, save, reset } =
    useConfigSection<RemoteDesktopConfig>(fetchFn, saveFn, { autoSave: false });

  const handleSave = async () => {
    const ok = await save();
    if (ok) {
      addToast('success', t('remoteDesktop.saved'));
    }
  };

  const handleRequestPermission = async () => {
    setRequesting(true);
    try {
      const result = await requestRemoteDesktopPermission();
      setPermissionGranted(result.granted);
      if (result.granted) {
        addToast('success', t('remoteDesktop.permissionRequested'));
      } else {
        addToast(
          'info',
          result.message || t('remoteDesktop.permissionRequestFailed'),
        );
      }
    } catch {
      addToast('error', t('remoteDesktop.permissionRequestFailed'));
    } finally {
      setRequesting(false);
    }
  };

  // Linux: no dialog — forget the portal's remembered display selection
  // (restore tokens); the next client connection re-asks which display to
  // share.
  const handleResetPermission = async () => {
    setRequesting(true);
    try {
      const result = await resetScreenConsent();
      if (result.error) {
        addToast('error', result.error);
      } else {
        setPermissionGranted(null);
        addToast('info', t('remoteDesktop.consent.resetDone', { count: result.cleared }));
      }
    } catch (err) {
      addToast('error', err instanceof Error ? err.message : String(err));
    } finally {
      setRequesting(false);
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
        <span className="font-light">{loadError ?? t('remoteDesktop.loadError')}</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SectionHeader title={t('remoteDesktop.title')} description={t('remoteDesktop.description')} />

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <ToggleField
          enabled={data.enabled}
          onChange={(v) => updateField('enabled', v)}
          label={t('remoteDesktop.enable')}
        />

        {data.enabled && (
          <p className="text-xs font-light text-warm-khaki">
            {t('remoteDesktop.enabledHint')}
          </p>
        )}

        {data.requiresPermission && (
          <div className="rounded-standard border border-golden-sand bg-bg-inset p-3 space-y-3">
            <div>
              <div className="text-sm font-semibold text-warm-black">
                {data.platform === 'macos'
                  ? t('remoteDesktop.permissionTitleMacos')
                  : t('remoteDesktop.permissionTitle')}
              </div>
              <p className="mt-1 text-xs font-light text-warm-khaki">
                {data.platform === 'linux'
                  ? t('remoteDesktop.consent.description')
                  : data.platform === 'macos'
                    ? t('remoteDesktop.permissionDescMacos')
                    : t('remoteDesktop.permissionDesc')}
              </p>
            </div>
            <div className="flex items-center justify-between gap-3">
              {data.platform === 'linux' ? (
                <>
                  <span className="text-xs font-light text-warm-khaki">
                    {t('remoteDesktop.consent.title')}
                  </span>
                  <button
                    type="button"
                    onClick={handleResetPermission}
                    disabled={requesting}
                    className="inline-flex items-center gap-2 px-4 py-1.5 rounded-standard text-xs font-semibold bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {requesting && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
                    {t('remoteDesktop.resetPermission')}
                  </button>
                </>
              ) : (
                <>
                  <PermissionStatus granted={permissionGranted} t={t} />
                  <button
                    type="button"
                    onClick={handleRequestPermission}
                    disabled={requesting}
                    className="inline-flex items-center gap-2 px-4 py-1.5 rounded-standard text-xs font-semibold bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {requesting && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
                    {t('remoteDesktop.grantPermission')}
                  </button>
                </>
              )}
            </div>
          </div>
        )}

        {data.platform === 'windows' && (
          <ServiceCard t={t} hostArch={data.arch} onResult={(kind, msg) => addToast(kind, msg)} />
        )}

        {data.platform === 'linux' && (
          <UinputCard t={t} onToast={(kind, msg) => addToast(kind, msg)} />
        )}

        <ToolPathsForm t={t} onSaved={(msg) => addToast('success', msg)} />

        <StreamSettingsForm t={t} onSaved={(msg) => addToast('success', msg)} />
      </div>

      <SaveResetButtons
        onSave={handleSave}
        onReset={reset}
        saving={saving}
        isDirty={isDirty}
      />

      {saveError && <div className="text-xs text-vt-error">{saveError}</div>}
    </div>
  );
}

const pathInputClassName = [
  'w-full bg-golden-cream px-2 py-1.5 text-sm font-normal',
  'rounded-standard border border-golden-sand',
  'transition-colors duration-vt-base',
  'focus:outline-none focus:border-amber-gold',
  'disabled:opacity-50 disabled:cursor-not-allowed',
  'text-warm-black',
].join(' ');

/// Windows service management: install/uninstall the auto-start
/// `VTermScreenSvc` service (the webui's server is a Windows host). Install
/// and uninstall run an elevated helper on the server — the user sees a UAC
/// prompt — so the requests block until the helper exits.

/// Virtual Display Driver release downloads (MikeTheTech's fork of the
/// Microsoft IddSampleDriver, MIT licensed): keeps a capture-able desktop
/// alive when no monitor is attached or the display sleeps.
const VDD_DOWNLOADS = {
  arm64: 'https://github.com/VirtualDrivers/Virtual-Display-Driver/releases/download/25.7.23/VirtualDisplayDriver-ARM64.Driver.Only.zip',
  x86: 'https://github.com/VirtualDrivers/Virtual-Display-Driver/releases/download/25.7.23/VirtualDisplayDriver-x86.Driver.Only.zip',
} as const;
const VDD_LICENSE_URL = 'https://github.com/VirtualDrivers/Virtual-Display-Driver/blob/master/LICENSE';

/// The download matching the host's CPU. `hostArch` comes from the server
/// (`std::env::consts::ARCH`) — the webui may run in a remote browser, so
/// the UA cannot be trusted. ARM64 Windows reports "aarch64"; everything
/// else takes the x86/x64 build.
function vddArch(hostArch: string | undefined): keyof typeof VDD_DOWNLOADS {
  return hostArch === 'aarch64' ? 'arm64' : 'x86';
}

function ServiceCard({
  t,
  hostArch,
  onResult,
}: {
  t: (key: string, params?: Record<string, string | number>) => string;
  /** Server-reported host CPU architecture (`data.arch`). */
  hostArch: string | undefined;
  onResult: (kind: 'success' | 'error' | 'info', message: string) => void;
}) {
  const [status, setStatus] = useState<ScreenServiceResponse | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setStatus(await fetchScreenService());
    } catch {
      setStatus(null);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const run = async (action: 'install' | 'remove') => {
    setBusy(true);
    try {
      const result = action === 'install'
        ? await installScreenService()
        : await removeScreenService();
      setStatus(result);
      if (result.message) {
        onResult('success', result.message);
      }
    } catch (err) {
      onResult('error', err instanceof Error ? err.message : String(err));
      await refresh();
    } finally {
      setBusy(false);
    }
  };

  const badge = (() => {
    if (!status) return { icon: ShieldQuestion, text: t('remoteDesktop.service.statusUnknown') };
    if (status.running) return { icon: ShieldCheck, text: t('remoteDesktop.service.statusRunning') };
    if (status.installed) return { icon: ShieldCheck, text: t('remoteDesktop.service.statusInstalled') };
    return { icon: ShieldAlert, text: t('remoteDesktop.service.statusNotInstalled') };
  })();
  const BadgeIcon = badge.icon;

  return (
    <div className="rounded-standard border border-golden-sand bg-bg-inset p-3 space-y-3">
      <div>
        <div className="text-sm font-semibold text-warm-black">
          {t('remoteDesktop.service.title')}
        </div>
        <p className="mt-1 text-xs font-light text-warm-khaki">
          {t('remoteDesktop.service.description')}
        </p>
      </div>
      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2 text-xs font-light text-warm-khaki">
          <BadgeIcon className="w-4 h-4 shrink-0" />
          <span>{badge.text}</span>
        </div>
        <div className="flex items-center gap-2">
          {status?.installed ? (
            <button
              type="button"
              onClick={() => run('remove')}
              disabled={busy}
              className="inline-flex items-center gap-2 px-4 py-1.5 rounded-standard text-xs font-semibold bg-bg-inset text-warm-black border border-golden-sand hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {busy && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
              {t('remoteDesktop.service.uninstall')}
            </button>
          ) : (
            <button
              type="button"
              onClick={() => run('install')}
              disabled={busy}
              className="inline-flex items-center gap-2 px-4 py-1.5 rounded-standard text-xs font-semibold bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {busy && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
              {t('remoteDesktop.service.install')}
            </button>
          )}
        </div>
      </div>
      {busy && (
        <p className="text-xs font-light text-warm-khaki">
          {t('remoteDesktop.service.elevating')}
        </p>
      )}
      {/* Headless hosts need the virtual display driver before the service
          is useful: with no monitor (or a sleeping one) Windows tears the
          desktop down and there is nothing to capture. Already-installed
          drivers collapse this to a status line. */}
      <div className="rounded-standard border border-golden-sand/60 p-2.5 space-y-1.5">
        <div className="flex items-center gap-1.5 text-xs font-semibold text-warm-black">
          <MonitorOff className="w-3.5 h-3.5 shrink-0" />
          {t('remoteDesktop.service.vdd.title')}
        </div>
        {status?.vddInstalled === true ? (
          <p className="flex items-center gap-1.5 text-xs font-light text-warm-khaki">
            <ShieldCheck className="w-3.5 h-3.5 shrink-0" />
            {t('remoteDesktop.service.vdd.installed')}
          </p>
        ) : (
          <>
            <p className="text-xs font-light text-warm-khaki">
              {t('remoteDesktop.service.vdd.description')}
            </p>
            <ul className="list-disc pl-4 space-y-0.5 text-xs font-light text-warm-khaki">
              <li>
                <a
                  href={VDD_DOWNLOADS[vddArch(hostArch)]}
                  target="_blank"
                  rel="noreferrer"
                  className="underline underline-offset-2 hover:opacity-80"
                >
                  {t('remoteDesktop.service.vdd.download', { arch: vddArch(hostArch) === 'arm64' ? 'ARM64' : 'x86' })}
                </a>
              </li>
              <li>{t('remoteDesktop.service.vdd.step')}</li>
              <li>{t('remoteDesktop.service.vdd.stepLegacy')}</li>
              <li>
                {t('remoteDesktop.service.vdd.stepSettings')}{' '}
                <code className="font-mono text-[11px] break-all">
                  copy vdd_settings.xml C:\Windows\System32\drivers\UMDF\
                </code>
              </li>
              <li>
                {t('remoteDesktop.service.vdd.license')}{' '}
                <a
                  href={VDD_LICENSE_URL}
                  target="_blank"
                  rel="noreferrer"
                  className="underline underline-offset-2 hover:opacity-80"
                >
                  MIT License
                </a>
              </li>
            </ul>
          </>
        )}
      </div>
    </div>
  );
}

/// The one-time Linux input grant the uinput card hands to the user
/// (POSIX sh, sudo): load the kernel module, give the `input` group
/// read-write access to `/dev/uinput`, and put the server's user in that
/// group. After a re-login the screen server can open the device and
/// inject input as a kernel virtual device — no portal consent dialog.
const UINPUT_SETUP_SCRIPT = `#!/bin/sh
# VTerm remote-input setup - run ONCE on this Linux host with sudo.
# Lets vterm-screen-server inject input through a kernel virtual device
# (uinput), so remote control no longer needs the "Remote Desktop"
# portal consent dialog.
set -e

# 1) Load the uinput kernel module now and at boot.
sudo modprobe uinput
echo uinput | sudo tee /etc/modules-load.d/vterm-uinput.conf >/dev/null

# 2) Let the "input" group open /dev/uinput read-write.
echo 'KERNEL=="uinput", MODE="0660", GROUP="input"' \\
  | sudo tee /etc/udev/rules.d/60-vterm-uinput.rules >/dev/null
sudo udevadm control --reload-rules
sudo udevadm trigger --name-match=uinput

# 3) Add this user to the input group (applies after logging out/in).
sudo usermod -aG input "$USER"

echo "Done. Log out and back in, restart the screen server, then re-check."
`;

/// Linux input-injection setup: the screen server probes whether it can
/// open `/dev/uinput` (the consent-free injection route). While it cannot,
/// the card shows the one-time setup script for the user to copy and run,
/// plus a re-check button to confirm the grant afterwards.
function UinputCard({
  t,
  onToast,
}: {
  t: (key: string, params?: Record<string, string | number>) => string;
  onToast: (kind: 'success' | 'error' | 'info', message: string) => void;
}) {
  const [status, setStatus] = useState<ScreenInputStatusResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  // `onToast` is deliberately not a dependency: it is only used from the
  // copy handler, so its identity never re-triggers the probe.
  const refresh = useCallback(async () => {
    setBusy(true);
    try {
      setStatus(await fetchScreenInputStatus());
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const copyScript = async () => {
    try {
      await navigator.clipboard.writeText(UINPUT_SETUP_SCRIPT);
      onToast('success', t('remoteDesktop.uinput.copied'));
    } catch {
      onToast('error', t('remoteDesktop.uinput.copyFailed'));
    }
  };

  const accessible = status?.uinputAccessible === true && !status.error;
  const badge = (() => {
    if (!status || status.error) {
      return { icon: ShieldQuestion, text: t('remoteDesktop.uinput.statusUnknown') };
    }
    if (accessible) {
      return { icon: ShieldCheck, text: t('remoteDesktop.uinput.statusReady') };
    }
    if (!status.uinputExists) {
      return { icon: ShieldAlert, text: t('remoteDesktop.uinput.statusNoDevice') };
    }
    return { icon: ShieldAlert, text: t('remoteDesktop.uinput.statusNoAccess') };
  })();
  const BadgeIcon = badge.icon;

  return (
    <div className="rounded-standard border border-golden-sand bg-bg-inset p-3 space-y-3">
      <div>
        <div className="text-sm font-semibold text-warm-black">
          {t('remoteDesktop.uinput.title')}
        </div>
        <p className="mt-1 text-xs font-light text-warm-khaki">
          {t('remoteDesktop.uinput.description')}
        </p>
      </div>

      <div className="flex items-center justify-between gap-3">
        <div className="flex items-center gap-2 text-xs font-light text-warm-khaki min-w-0">
          <BadgeIcon className="w-4 h-4 shrink-0" />
          <span className="truncate">{badge.text}</span>
        </div>
        <button
          type="button"
          onClick={() => void refresh()}
          disabled={busy}
          className="inline-flex items-center gap-2 px-4 py-1.5 rounded-standard text-xs font-semibold bg-bg-inset text-warm-black border border-golden-sand hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {busy && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
          {t('remoteDesktop.uinput.recheck')}
        </button>
      </div>

      {error && <div className="text-xs text-vt-error break-all">{error}</div>}
      {status?.error && (
        <div className="text-xs text-vt-error break-all">{status.error}</div>
      )}

      {!accessible && (
        <div className="space-y-2">
          <ol className="list-decimal pl-4 space-y-0.5 text-xs font-light text-warm-khaki">
            <li>{t('remoteDesktop.uinput.step1')}</li>
            <li>{t('remoteDesktop.uinput.step2')}</li>
            <li>{t('remoteDesktop.uinput.step3')}</li>
          </ol>
          <div className="flex justify-end">
            <button
              type="button"
              onClick={() => void copyScript()}
              className="inline-flex items-center gap-2 px-4 py-1.5 rounded-standard text-xs font-semibold bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90 transition-opacity duration-vt-base"
            >
              <Copy className="w-3.5 h-3.5" />
              {t('remoteDesktop.uinput.copyScript')}
            </button>
          </div>
          <pre className="overflow-x-auto rounded-standard border border-golden-sand/60 bg-golden-cream p-2.5 text-[11px] leading-relaxed font-mono text-warm-black whitespace-pre">
            {UINPUT_SETUP_SCRIPT}
          </pre>
          <p className="text-xs font-light text-warm-khaki">
            {t('remoteDesktop.uinput.note')}
          </p>
        </div>
      )}
    </div>
  );
}

/// Screen-streaming tool paths (spec §4.3). This form reads from two sources —
/// the persisted overrides (`configured`) and the screen-server's resolved
/// state — so it keeps its own local state instead of `useConfigSection`.
function ToolPathsForm({
  t,
  onSaved,
}: {
  t: (key: string, params?: Record<string, string | number>) => string;
  onSaved: (message: string) => void;
}) {
  const [sdkInput, setSdkInput] = useState('');
  const [agentInput, setAgentInput] = useState('');
  const [resolved, setResolved] = useState<ScreenPathsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    fetchScreenPaths()
      .then((response) => {
        if (cancelled) return;
        setResolved(response);
        setSdkInput(response.configured.androidSdkPath ?? '');
        setAgentInput(response.configured.androidStudioPath ?? '');
      })
      .catch((err: Error) => {
        if (!cancelled) setError(err.message);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const submit = async (androidSdkPath: string | null, androidStudioPath: string | null) => {
    setSaving(true);
    setError(null);
    try {
      const response = await saveScreenPaths({ androidSdkPath, androidStudioPath });
      setResolved(response);
      setSdkInput(response.configured.androidSdkPath ?? '');
      setAgentInput(response.configured.androidStudioPath ?? '');
      if (response.pushError) {
        setError(response.pushError);
      } else {
        onSaved(t('remoteDesktop.pathsSaved'));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center gap-2 text-warm-khaki">
        <Loader2 className="w-4 h-4 animate-spin" />
        <span className="text-xs font-light">{t('remoteDesktop.pathsLoading')}</span>
      </div>
    );
  }

  return (
    <div className="rounded-standard border border-golden-sand bg-bg-inset p-3 space-y-3">
      <div>
        <div className="text-sm font-semibold text-warm-black">
          {t('remoteDesktop.toolPathsTitle')}
        </div>
        <p className="mt-1 text-xs font-light text-warm-khaki">
          {t('remoteDesktop.toolPathsDesc')}
        </p>
      </div>

      {/* The screen-server being unreachable is a service problem (Windows:
          the VTermScreenSvc service), not an Android-path problem — show it
          once here instead of as an error under each path input. */}
      {resolved?.error && (
        <div className="text-xs font-normal text-vt-error break-all">
          {t('remoteDesktop.screenServerUnreachable', { error: resolved.error })}
        </div>
      )}

      <div className="space-y-1.5">
        <label htmlFor="screen-android-sdk-path" className="block text-xs font-medium text-warm-black">
          {t('remoteDesktop.androidSdkPath')}
        </label>
        <input
          id="screen-android-sdk-path"
          type="text"
          value={sdkInput}
          onChange={(e) => setSdkInput(e.target.value)}
          placeholder={t('remoteDesktop.androidSdkPathHint')}
          disabled={saving}
          className={pathInputClassName}
          spellCheck={false}
        />
        {!resolved?.error && <ResolvedPathLine info={resolved?.sdk} t={t} />}
      </div>

      <div className="space-y-1.5">
        <label htmlFor="screen-agent-dir" className="block text-xs font-medium text-warm-black">
          {t('remoteDesktop.agentDirPath')}
        </label>
        <input
          id="screen-agent-dir"
          type="text"
          value={agentInput}
          onChange={(e) => setAgentInput(e.target.value)}
          placeholder={t('remoteDesktop.agentDirPathHint')}
          disabled={saving}
          className={pathInputClassName}
          spellCheck={false}
        />
        {!resolved?.error && <ResolvedPathLine info={resolved?.agentDir} t={t} />}
      </div>

      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={() => submit(sdkInput.trim() || null, agentInput.trim() || null)}
          disabled={saving}
          className="inline-flex items-center gap-2 px-4 py-1.5 rounded-standard text-xs font-semibold bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {saving && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
          {t('remoteDesktop.save')}
        </button>
        <button
          type="button"
          onClick={() => {
            setSdkInput('');
            setAgentInput('');
            void submit(null, null);
          }}
          disabled={saving}
          className="inline-flex items-center gap-2 px-4 py-1.5 rounded-standard text-xs font-semibold bg-transparent text-warm-black border border-golden-sand hover:bg-pale-sand transition-colors duration-vt-base disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {t('remoteDesktop.resetToAuto')}
        </button>
      </div>

      {error && <div className="text-xs text-vt-error break-all">{error}</div>}
    </div>
  );
}

const FPS_OPTIONS = [60, 30, 15];

/// Max-resolution ceiling for the capture (virtual-display size on macOS);
/// ids match the ctl `SetStreamConfig.max_resolution` wire values.
const MAX_RESOLUTION_OPTIONS = [
  { id: '4k', label: '4K (3840×2160)' },
  { id: '2.5k', label: '2.5K (2560×1440)' },
  { id: '2k', label: '2K (1920×1080)' },
];
const DEFAULT_MAX_RESOLUTION = '4k';

/// Fixed-bitrate options (Mbps); must match the ctl `BITRATE_MBPS_OPTIONS`.
/// null/auto = the server area-scales the configured base by resolution.
const BITRATE_OPTIONS: Array<number | null> = [
  null, 50, 45, 40, 35, 30, 25, 20, 15, 10, 5, 3,
];

/// Screen-streaming encoder / frame-rate settings. Options come from the
/// screen-server's availability catalog; unavailable backends are shown
/// disabled with their reason. Same dual-source pattern as ToolPathsForm
/// (persisted `configured` + live screen-server state), hence local state.
function StreamSettingsForm({
  t,
  onSaved,
}: {
  t: (key: string, params?: Record<string, string | number>) => string;
  onSaved: (message: string) => void;
}) {
  const [encoderId, setEncoderId] = useState('auto');
  const [fps, setFps] = useState(60);
  const [maxResolution, setMaxResolution] = useState(DEFAULT_MAX_RESOLUTION);
  const [bitrateMbps, setBitrateMbps] = useState<number | null>(null);
  const [resolved, setResolved] = useState<ScreenEncoderResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    fetchScreenEncoder()
      .then((response) => {
        if (cancelled) return;
        setResolved(response);
        setEncoderId(response.configured.encoder ?? 'auto');
        setFps(response.configured.fps ?? 60);
        setMaxResolution(response.configured.maxResolution ?? DEFAULT_MAX_RESOLUTION);
        setBitrateMbps(
          response.configured.bitrateMbps != null &&
            BITRATE_OPTIONS.includes(response.configured.bitrateMbps)
            ? response.configured.bitrateMbps
            : null,
        );
      })
      .catch((err: Error) => {
        if (!cancelled) setError(err.message);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const submit = async () => {
    setSaving(true);
    setError(null);
    try {
      const response = await saveScreenEncoder({
        encoder: encoderId === 'auto' ? null : encoderId,
        fps,
        maxResolution,
        bitrateMbps,
        // Not user-configurable: the macOS backend must always capture
        // through the CGVirtualDisplay path.
        virtualDisplay: true,
      });
      setResolved(response);
      setEncoderId(response.configured.encoder ?? 'auto');
      setFps(response.configured.fps ?? 60);
      setMaxResolution(response.configured.maxResolution ?? DEFAULT_MAX_RESOLUTION);
      setBitrateMbps(response.configured.bitrateMbps ?? null);
      if (response.pushError) {
        setError(response.pushError);
      } else {
        onSaved(t('remoteDesktop.streamSaved'));
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center gap-2 text-warm-khaki">
        <Loader2 className="w-4 h-4 animate-spin" />
        <span className="text-xs font-light">{t('remoteDesktop.streamLoading')}</span>
      </div>
    );
  }

  return (
    <div className="rounded-standard border border-golden-sand bg-bg-inset p-3 space-y-3">
      <div>
        <div className="text-sm font-semibold text-warm-black">
          {t('remoteDesktop.streamTitle')}
        </div>
        <p className="mt-1 text-xs font-light text-warm-khaki">
          {t('remoteDesktop.streamDesc')}
        </p>
      </div>

      <div className="space-y-1.5">
        <label className="block text-xs font-medium text-warm-black">
          {t('remoteDesktop.encoder')}
        </label>
        <SelectField
          value={encoderId}
          onChange={(e) => setEncoderId(e.target.value)}
          disabled={saving}
          ariaLabel={t('remoteDesktop.encoder')}
          options={(resolved?.entries ?? []).map((entry) => ({
            value: entry.id,
            label: entry.available || !entry.reason ? entry.label : `${entry.label} — ${entry.reason}`,
            disabled: !entry.available,
          }))}
        />
      </div>

      <div className="space-y-1.5">
        <label className="block text-xs font-medium text-warm-black">
          {t('remoteDesktop.frameRate')}
        </label>
        <SelectField
          value={String(fps)}
          onChange={(e) => setFps(Number(e.target.value))}
          disabled={saving}
          ariaLabel={t('remoteDesktop.frameRate')}
          options={FPS_OPTIONS.map((value) => ({ value: String(value), label: `${value} fps` }))}
        />
      </div>

      <div className="space-y-1.5">
        <label className="block text-xs font-medium text-warm-black">
          {t('remoteDesktop.maxResolution')}
        </label>
        <SelectField
          value={maxResolution}
          onChange={(e) => setMaxResolution(e.target.value)}
          disabled={saving}
          ariaLabel={t('remoteDesktop.maxResolution')}
          options={MAX_RESOLUTION_OPTIONS.map((option) => ({ value: option.id, label: option.label }))}
        />
      </div>

      <div className="space-y-1.5">
        <label className="block text-xs font-medium text-warm-black">
          {t('remoteDesktop.bitrate')}
        </label>
        <SelectField
          value={bitrateMbps == null ? 'auto' : String(bitrateMbps)}
          onChange={(e) => {
            const v = e.target.value;
            setBitrateMbps(v === 'auto' ? null : Number(v));
          }}
          disabled={saving}
          ariaLabel={t('remoteDesktop.bitrate')}
          options={BITRATE_OPTIONS.map((value) => ({
            value: value == null ? 'auto' : String(value),
            label: value == null ? t('remoteDesktop.bitrateAuto') : `${value} Mbps`,
          }))}
        />
        <p className="text-xs font-light text-warm-khaki">
          {t('remoteDesktop.bitrateHint')}
        </p>
      </div>

      {resolved?.error ? (
        <p className="text-xs font-normal text-vt-error break-all">{resolved.error}</p>
      ) : (
        resolved?.current != null &&
        resolved?.fps != null && (
          <p className="text-xs font-normal text-warm-khaki">
            {t('remoteDesktop.streamInEffect', {
              encoder: resolved.current,
              fps: resolved.fps,
              maxResolution: resolved.maxResolution ?? DEFAULT_MAX_RESOLUTION,
              bitrate:
                resolved.bitrateKbps != null
                  ? `${resolved.bitrateKbps / 1000} Mbps`
                  : t('remoteDesktop.bitrateAuto'),
            })}
          </p>
        )
      )}

      <div className="flex items-center gap-2">
        <button
          type="button"
          onClick={() => void submit()}
          disabled={saving}
          className="inline-flex items-center gap-2 px-4 py-1.5 rounded-standard text-xs font-semibold bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90 transition-opacity duration-vt-base disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {saving && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
          {t('remoteDesktop.save')}
        </button>
      </div>

      {error && <div className="text-xs text-vt-error break-all">{error}</div>}
    </div>
  );
}

function ResolvedPathLine({
  info,
  t,
}: {
  info: ScreenPathInfo | undefined;
  t: (key: string, params?: Record<string, string | number>) => string;
}) {
  if (!info) {
    return null;
  }
  if (info.valid && info.value) {
    return (
      <p className="text-xs font-normal text-warm-khaki break-all">
        {t('remoteDesktop.resolvedValue', { path: info.value, origin: info.origin })}
      </p>
    );
  }
  return (
    <p className="text-xs font-normal text-vt-error break-all">
      {info.detail || t('remoteDesktop.resolvedNotFound')}
    </p>
  );
}

function PermissionStatus({
  granted,
  t,
}: {
  granted: boolean | null;
  t: (key: string, params?: Record<string, string | number>) => string;
}) {
  if (granted === true) {
    return (
      <span className="inline-flex items-center gap-1.5 text-xs font-medium text-amber-gold">
        <ShieldCheck className="w-4 h-4" />
        {t('remoteDesktop.permissionGranted')}
      </span>
    );
  }
  if (granted === false) {
    return (
      <span className="inline-flex items-center gap-1.5 text-xs font-medium text-warm-khaki">
        <ShieldAlert className="w-4 h-4" />
        {t('remoteDesktop.permissionNotGranted')}
      </span>
    );
  }
  return (
    <span className="inline-flex items-center gap-1.5 text-xs font-medium text-warm-khaki">
      <ShieldQuestion className="w-4 h-4" />
      {t('remoteDesktop.permissionUnknown')}
    </span>
  );
}
