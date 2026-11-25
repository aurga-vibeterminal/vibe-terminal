import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  AlertCircle,
  CheckCircle,
  ChevronDown,
  ChevronUp,
  Circle,
  Download,
  ExternalLink,
  Loader2,
  Mic,
  RefreshCw,
  Sparkles,
  Square,
  Radio,
  Zap,
} from 'lucide-react';
import {
  downloadLocalAsrModel,
  getVoiceConfig,
  getVoiceModelStatus,
  polishVoiceText,
  scanLocalAsrModels,
  startVoiceServer,
  stopVoiceServer,
  transcribeFile,
  updateVoiceConfig,
} from '@/api/voice';
import { getAddons } from '@/api/addon';
import { getLlmConfig, updateLlmConfig } from '@/api/llm';
import { CopyButton } from '@/components/shared/CopyButton';
import { FormField } from '@/components/shared/FormField';
import { SecretField } from '@/components/shared/SecretField';
import { SelectField } from '@/components/shared/SelectField';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { ToggleField } from '@/components/shared/ToggleField';
import { useConfigSection } from '@/hooks/use-config-section';
import { useLiveTranscriber } from '@/hooks/useLiveTranscriber';
import { useTranslation } from '@/i18n';
import type {
  VoiceConfig,
  AsrServerConfig,
  AsrModelStatus,
  AddonInfo,
  LlmConfig,
  LlmProviderConfig,
  LocalAsrModelAvailability,
  LocalAsrModelSource,
} from '@/types';
import { renderAddonConfigField, resolveAddonConfig } from '@/lib/addon-config';
import { generateRandomToken } from '@/lib/random-token';

const inputClassName = [
  'w-full bg-bg-inset px-2 py-1.5 text-sm font-normal',
  'rounded-standard border border-golden-sand',
  'transition-colors duration-vt-base',
  'focus:outline-none focus:border-amber-gold',
  'disabled:opacity-50 disabled:cursor-not-allowed',
  'text-warm-black',
].join(' ');

type LocalAsrModelDefinition = {
  id: string;
  label: string;
  size: string;
  family: string;
  languages: string;
  huggingFace: string;
  modelScope?: string;
};

const LOCAL_ASR_MODELS: LocalAsrModelDefinition[] = [
  {
    id: 'qwen3-asr-0.6b', label: 'Qwen3-ASR-0.6B', size: '1.75 GB', family: 'Qwen3-ASR',
    languages: 'voice.modelLanguagesQwen',
    huggingFace: 'https://huggingface.co/Qwen/Qwen3-ASR-0.6B', modelScope: 'https://modelscope.cn/models/Qwen/Qwen3-ASR-0.6B',
  },
  {
    id: 'qwen3-asr-0.6b-8bit', label: 'Qwen3-ASR-0.6B · 8-bit', size: '0.96 GB', family: 'Qwen3-ASR',
    languages: 'voice.modelLanguagesQwen',
    huggingFace: 'https://huggingface.co/mlx-community/Qwen3-ASR-0.6B-8bit', modelScope: 'https://modelscope.cn/models/mlx-community/Qwen3-ASR-0.6B-8bit',
  },
  {
    id: 'qwen3-asr-0.6b-4bit', label: 'Qwen3-ASR-0.6B · 4-bit', size: '0.68 GB', family: 'Qwen3-ASR',
    languages: 'voice.modelLanguagesQwen',
    huggingFace: 'https://huggingface.co/mlx-community/Qwen3-ASR-0.6B-4bit', modelScope: 'https://modelscope.cn/models/mlx-community/Qwen3-ASR-0.6B-4bit',
  },
  {
    id: 'qwen3-asr-1.7b', label: 'Qwen3-ASR-1.7B', size: '4.37 GB', family: 'Qwen3-ASR',
    languages: 'voice.modelLanguagesQwen',
    huggingFace: 'https://huggingface.co/Qwen/Qwen3-ASR-1.7B', modelScope: 'https://modelscope.cn/models/Qwen/Qwen3-ASR-1.7B',
  },
  {
    id: 'qwen3-asr-1.7b-8bit', label: 'Qwen3-ASR-1.7B · 8-bit', size: '2.29 GB', family: 'Qwen3-ASR',
    languages: 'voice.modelLanguagesQwen',
    huggingFace: 'https://huggingface.co/mlx-community/Qwen3-ASR-1.7B-8bit', modelScope: 'https://modelscope.cn/models/mlx-community/Qwen3-ASR-1.7B-8bit',
  },
  {
    id: 'qwen3-asr-1.7b-4bit', label: 'Qwen3-ASR-1.7B · 4-bit', size: '1.49 GB', family: 'Qwen3-ASR',
    languages: 'voice.modelLanguagesQwen',
    huggingFace: 'https://huggingface.co/mlx-community/Qwen3-ASR-1.7B-4bit', modelScope: 'https://modelscope.cn/models/mlx-community/Qwen3-ASR-1.7B-4bit',
  },
  {
    id: 'fun-asr-nano-2512', label: 'Fun-ASR-Nano-2512', size: '1.99 GB', family: 'Fun-ASR',
    languages: 'voice.modelLanguagesFun',
    huggingFace: 'https://huggingface.co/FunAudioLLM/Fun-ASR-Nano-2512', modelScope: 'https://modelscope.cn/models/FunAudioLLM/Fun-ASR-Nano-2512',
  },
  {
    id: 'fun-asr-nano-2512-8bit', label: 'Fun-ASR-Nano-2512 · 8-bit', size: '1.46 GB', family: 'Fun-ASR',
    languages: 'voice.modelLanguagesFun',
    huggingFace: 'https://huggingface.co/mlx-community/Fun-ASR-Nano-2512-8bit', modelScope: 'https://modelscope.cn/models/mlx-community/Fun-ASR-Nano-2512-8bit',
  },
  {
    id: 'fun-asr-nano-2512-4bit', label: 'Fun-ASR-Nano-2512 · 4-bit', size: '1.24 GB', family: 'Fun-ASR',
    languages: 'voice.modelLanguagesFun',
    huggingFace: 'https://huggingface.co/mlx-community/Fun-ASR-Nano-2512-4bit', modelScope: 'https://modelscope.cn/models/mlx-community/Fun-ASR-Nano-2512-4bit',
  },
  {
    id: 'fun-asr-mlt-nano-2512', label: 'Fun-ASR-MLT-Nano-2512', size: '1.85 GB', family: 'Fun-ASR MLT',
    languages: 'voice.modelLanguagesFunMlt',
    huggingFace: 'https://huggingface.co/FunAudioLLM/Fun-ASR-MLT-Nano-2512', modelScope: 'https://modelscope.cn/models/FunAudioLLM/Fun-ASR-MLT-Nano-2512',
  },
  {
    id: 'fun-asr-mlt-nano-2512-8bit', label: 'Fun-ASR-MLT-Nano-2512 · 8-bit', size: '1.45 GB', family: 'Fun-ASR MLT',
    languages: 'voice.modelLanguagesFunMlt',
    huggingFace: 'https://huggingface.co/mlx-community/Fun-ASR-MLT-Nano-2512-8bit', modelScope: 'https://modelscope.cn/models/mlx-community/Fun-ASR-MLT-Nano-2512-8bit',
  },
  {
    id: 'fun-asr-mlt-nano-2512-4bit', label: 'Fun-ASR-MLT-Nano-2512 · 4-bit', size: '1.24 GB', family: 'Fun-ASR MLT',
    languages: 'voice.modelLanguagesFunMlt',
    huggingFace: 'https://huggingface.co/mlx-community/Fun-ASR-MLT-Nano-2512-4bit', modelScope: 'https://modelscope.cn/models/mlx-community/Fun-ASR-MLT-Nano-2512-4bit',
  },
  {
    id: 'parakeet-tdt-0.6b-v3', label: 'Parakeet-TDT-0.6B-V3', size: '2.33 GB', family: 'NVIDIA Parakeet',
    languages: 'voice.modelLanguagesParakeet',
    huggingFace: 'https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3', modelScope: 'https://modelscope.cn/models/nv-community/parakeet-tdt-0.6b-v3',
  },
  {
    id: 'parakeet-tdt-0.6b-v3-8bit', label: 'Parakeet-TDT-0.6B-V3 · 8-bit', size: '0.72 GB', family: 'NVIDIA Parakeet',
    languages: 'voice.modelLanguagesParakeet',
    huggingFace: 'https://huggingface.co/sonic-speech/parakeet-tdt-0.6b-v3-int8',
  },
  {
    id: 'parakeet-tdt-0.6b-v3-4bit', label: 'Parakeet-TDT-0.6B-V3 · 4-bit', size: '0.47 GB', family: 'NVIDIA Parakeet',
    languages: 'voice.modelLanguagesParakeet',
    huggingFace: 'https://huggingface.co/sonic-speech/parakeet-tdt-0.6b-v3-int4',
  },
];

function ModelStatusBadge({ status, t }: { status: AsrModelStatus; t: (key: string) => string }) {
  const { status: state, ...rest } = status;
  switch (state) {
    case 'loaded':
      return (
        <span className="inline-flex items-center gap-1.5 text-xs px-2 py-1 rounded-standard bg-green-50 text-green-700 border border-green-200">
          <span className="w-1.5 h-1.5 rounded-full bg-green-500" />
          {t('voice.modelLoaded').replace('{model}', (rest as { modelId: string }).modelId).replace('{device}', (rest as { device: string }).device)}
        </span>
      );
    case 'loading':
      return (
        <span className="inline-flex items-center gap-1.5 text-xs px-2 py-1 rounded-standard bg-amber-50 text-amber-700 border border-amber-200">
          <Loader2 className="w-3 h-3 animate-spin" />
          {t('voice.modelLoading')}
        </span>
      );
    case 'error':
      return (
        <span className="inline-flex items-center gap-1.5 text-xs px-2 py-1 rounded-standard bg-red-50 text-red-700 border border-red-200">
          <AlertCircle className="w-3 h-3" />
          {t('voice.modelError').replace('{message}', (rest as { message: string }).message)}
        </span>
      );
    default:
      // "Not loaded" is the natural state before the user starts the server —
      // render nothing so we don't clutter the card with a gray idle badge.
      return null;
  }
}

// --- Collapsible Section ---
function CollapsibleCard({
  title,
  expanded,
  onToggle,
  statusIcon,
  children,
}: {
  title: string;
  expanded: boolean;
  onToggle: () => void;
  statusIcon?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <div className="rounded-comfortable border border-golden-sand bg-pale-sand/40">
      <div className="flex items-center justify-between p-4">
        <div className="flex items-center gap-3 flex-1 min-w-0">
          <button
            type="button"
            onClick={onToggle}
            className="p-1 rounded-standard text-deep-olive hover:text-warm-black hover:bg-golden-cream transition-colors duration-vt-base shrink-0"
            aria-label={expanded ? 'Collapse' : 'Expand'}
          >
            {expanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
          </button>
          <h4 className="text-sm font-semibold text-warm-black truncate">
            {title}
          </h4>
        </div>
        {statusIcon && (
          <div className="ml-3 shrink-0">{statusIcon}</div>
        )}
      </div>
      {expanded && (
        <div className="px-4 pb-4 pt-0 space-y-5 border-t border-golden-sand/60 mt-0 pt-4">
          {children}
        </div>
      )}
    </div>
  );
}

// --- Local ASR Status Icon (compact indicator for the Provider Config header) ---
function LocalAsrStatusIcon({
  reachable,
  modelStatus,
  t,
}: {
  reachable: boolean;
  modelStatus: AsrModelStatus;
  t: (key: string) => string;
}) {
  if (!reachable) {
    return (
      <span
        className="inline-flex items-center text-amber-600"
        title={t('voice.asrServerNotReachable')}
      >
        <AlertCircle className="w-4 h-4" />
      </span>
    );
  }
  switch (modelStatus.status) {
    case 'loaded': {
      const { modelId, device } = modelStatus;
      return (
        <span
          className="inline-flex items-center text-green-600"
          title={t('voice.modelLoaded').replace('{model}', modelId).replace('{device}', device)}
        >
          <CheckCircle className="w-4 h-4" />
        </span>
      );
    }
    case 'loading':
      return (
        <span
          className="inline-flex items-center text-amber-600"
          title={t('voice.modelLoading')}
        >
          <Loader2 className="w-4 h-4 animate-spin" />
        </span>
      );
    case 'error': {
      const { message } = modelStatus;
      return (
        <span
          className="inline-flex items-center text-red-600"
          title={t('voice.modelError').replace('{message}', message)}
        >
          <AlertCircle className="w-4 h-4" />
        </span>
      );
    }
    case 'not_loaded':
    default:
      return (
        <span
          className="inline-flex items-center text-gray-400"
          title={t('voice.modelNotLoaded')}
        >
          <Circle className="w-4 h-4" />
        </span>
      );
  }
}

function LocalAsrModelManager({
  directory,
  onDirectoryChange,
  selectedId,
  onModelChange,
  t,
}: {
  directory: string;
  onDirectoryChange: (directory: string) => void;
  selectedId: string;
  onModelChange: (id: string, relativePath: string) => void;
  t: (key: string) => string;
}) {
  const [availability, setAvailability] = useState<Record<string, LocalAsrModelAvailability>>({});
  const [scanning, setScanning] = useState(false);
  const [scanError, setScanError] = useState<string | null>(null);
  const [downloading, setDownloading] = useState<Record<string, boolean>>({});
  const [downloadSource, setDownloadSource] = useState<LocalAsrModelSource>('huggingface');

  const scan = useCallback(async () => {
    setScanning(true);
    setScanError(null);
    try {
      const result = await scanLocalAsrModels(directory);
      setAvailability(Object.fromEntries(result.models.map((model) => [model.id, model])));
    } catch (error) {
      // The endpoint is intentionally server-owned and may not exist until a
      // server version implementing model management is installed. Keep the
      // static catalog usable so users can still follow manual instructions.
      setScanError(error instanceof Error ? error.message : String(error));
    } finally {
      setScanning(false);
    }
  }, [directory]);

  useEffect(() => { void scan(); }, [scan]);

  const selected = LOCAL_ASR_MODELS.find((model) => model.id === selectedId) ?? LOCAL_ASR_MODELS[0];
  const license = selected.id.startsWith('parakeet-')
    ? {
        label: 'CC BY 4.0',
        url: 'https://huggingface.co/datasets/choosealicense/licenses/blob/main/markdown/cc-by-4.0.md',
      }
    : {
        label: 'Apache 2.0',
        url: 'https://huggingface.co/datasets/choosealicense/licenses/blob/main/markdown/apache-2.0.md',
      };
  useEffect(() => {
    if (!selected.modelScope && downloadSource === 'modelscope') {
      setDownloadSource('huggingface');
    }
  }, [downloadSource, selected.modelScope]);
  const selectedAvailability = availability[selected.id];
  const downloadState = selectedAvailability?.download;
  const isAvailable = selectedAvailability?.available === true;
  const isDownloading = downloading[selected.id] || downloadState?.status === 'queued' || downloadState?.status === 'downloading';

  const hasActiveDownload = Object.values(availability).some((model) =>
    model.download?.status === 'queued' || model.download?.status === 'downloading',
  );
  useEffect(() => {
    if (!hasActiveDownload) return;
    const interval = window.setInterval(() => { void scan(); }, 1500);
    return () => window.clearInterval(interval);
  }, [hasActiveDownload, scan]);

  const download = useCallback(async () => {
    setDownloading((current) => ({ ...current, [selected.id]: true }));
    try {
      const updated = await downloadLocalAsrModel(selected.id, directory, downloadSource);
      setAvailability((current) => ({ ...current, [selected.id]: updated }));
    } catch (error) {
      setAvailability((current) => ({
        ...current,
        [selected.id]: {
          id: selected.id,
          available: false,
          download: { status: 'failed', message: error instanceof Error ? error.message : String(error) },
        },
      }));
    } finally {
      setDownloading((current) => ({ ...current, [selected.id]: false }));
    }
  }, [directory, downloadSource, selected.id]);

  return (
    <div className="space-y-3">
      <FormField label={t('voice.modelDirectory')} description={t('voice.modelDirectoryDesc')}>
        <div className="flex gap-2">
          <input
            type="text"
            value={directory}
            onChange={(event) => onDirectoryChange(event.target.value)}
            onBlur={() => { void scan(); }}
            className={inputClassName}
            placeholder="/path/to/asr-models"
          />
          <button
            type="button"
            onClick={() => { void scan(); }}
            disabled={scanning}
            className="shrink-0 p-2 rounded-standard border border-golden-sand text-deep-olive hover:bg-pale-sand disabled:opacity-50"
            aria-label={t('voice.refreshModels')}
            title={t('voice.refreshModels')}
          >
            <RefreshCw className={`w-4 h-4 ${scanning ? 'animate-spin' : ''}`} />
          </button>
        </div>
      </FormField>

      <FormField label={t('voice.localModel')} description={t('voice.localModelDesc')}>
        <SelectField
          value={selected.id}
          onChange={(event) => onModelChange(event.target.value, event.target.value)}
          options={LOCAL_ASR_MODELS.map((model) => {
            const modelAvailability = availability[model.id];
            const progress = modelAvailability?.download?.progress;
            const isModelDownloading = downloading[model.id]
              || modelAvailability?.download?.status === 'queued'
              || modelAvailability?.download?.status === 'downloading';
            const status = modelAvailability?.available
              ? ` — ${t('voice.modelAvailable')}`
              : isModelDownloading
                ? ` — ${typeof progress === 'number' ? `${Math.round(progress)}%` : t('voice.modelDownloading')}`
                : '';
            return { value: model.id, label: `${model.label} (${model.size})${status}` };
          })}
        />
      </FormField>

      <div className="rounded-standard border border-golden-sand/80 bg-bg-inset/50 p-3 space-y-2 text-xs text-deep-olive">
        <div className="flex flex-wrap items-center gap-2">
          <span className="font-semibold text-warm-black">{selected.label}</span>
          {isAvailable ? (
            <span className="inline-flex items-center gap-1 text-green-700"><CheckCircle className="w-3.5 h-3.5" />{t('voice.modelAvailable')}</span>
          ) : isDownloading ? (
            <span className="inline-flex items-center gap-1 text-amber-700"><Loader2 className="w-3.5 h-3.5 animate-spin" />{t('voice.modelDownloading')}</span>
          ) : (
            <span className="inline-flex items-center gap-1 text-amber-700"><AlertCircle className="w-3.5 h-3.5" />{t('voice.modelUnavailable')}</span>
          )}
        </div>
        <p><span className="font-medium">{t('voice.modelLanguages')}:</span> {t(selected.languages)}</p>
        {selectedAvailability?.path && <p><span className="font-medium">{t('voice.modelDetectedAt')}:</span> {selectedAvailability.path}</p>}
        {isDownloading && typeof downloadState?.progress === 'number' && (
          <div className="space-y-1">
            <div className="h-1.5 rounded-full bg-golden-sand overflow-hidden">
              <div className="h-full bg-amber-gold transition-all" style={{ width: `${Math.max(0, Math.min(100, downloadState.progress))}%` }} />
            </div>
            <p>{t('voice.downloadProgress').replace('{progress}', String(Math.round(downloadState.progress)))}</p>
          </div>
        )}
        {downloadState?.status === 'failed' && <p className="text-vt-error">{downloadState.message ?? t('voice.downloadFailed')}</p>}
        <div className="flex flex-wrap gap-2 pt-1">
          {!isAvailable && (
            <div className="inline-flex gap-1">
              <div className="w-36">
                <SelectField
                  value={downloadSource}
                  onChange={(event) => setDownloadSource(event.target.value as LocalAsrModelSource)}
                  options={[
                    { value: 'huggingface', label: 'Hugging Face' },
                    ...(selected.modelScope ? [{ value: 'modelscope', label: 'ModelScope' }] : []),
                  ]}
                />
              </div>
              <button
                type="button"
                onClick={() => { void download(); }}
                disabled={isDownloading || !directory.trim()}
                className="inline-flex items-center gap-1.5 px-2.5 py-1.5 rounded-standard bg-accent-subtle text-accent-text hover:bg-accent-subtle/80 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isDownloading ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Download className="w-3.5 h-3.5" />}
                {isDownloading ? t('voice.modelDownloading') : t('voice.downloadModel')}
              </button>
            </div>
          )}
          <a className="inline-flex items-center gap-1 text-accent-text hover:underline" href={selected.huggingFace} target="_blank" rel="noreferrer">
            Hugging Face <ExternalLink className="w-3.5 h-3.5" />
          </a>
          {selected.modelScope && (
            <a className="inline-flex items-center gap-1 text-accent-text hover:underline" href={selected.modelScope} target="_blank" rel="noreferrer">
              ModelScope <ExternalLink className="w-3.5 h-3.5" />
            </a>
          )}
          <a className="inline-flex items-center gap-1 text-accent-text hover:underline" href={license.url} target="_blank" rel="noreferrer">
            License: {license.label} <ExternalLink className="w-3.5 h-3.5" />
          </a>
        </div>
        <p className="text-deep-olive/75">{t('voice.manualDownloadHint').replace('{directory}', directory || '/path/to/asr-models')}</p>
        <div className="space-y-1 text-deep-olive/75">
          <p>{t('voice.manualDownloadCommands')}</p>
          <code className="block break-all rounded bg-pale-sand px-2 py-1">git lfs install &amp;&amp; git clone {selected.huggingFace} &quot;{directory || '/path/to/asr-models'}/{selected.id}&quot;</code>
          {selected.modelScope && (
            <code className="block break-all rounded bg-pale-sand px-2 py-1">modelscope download --model {new URL(selected.modelScope).pathname.replace('/models/', '')} --local_dir &quot;{directory || '/path/to/asr-models'}/{selected.id}&quot;</code>
          )}
        </div>
        {scanError && <p className="text-amber-700">{t('voice.modelScanUnavailable')}</p>}
      </div>
    </div>
  );
}

// --- Local ASR Server Card ---
// Renders as a nested sub-section under the Enable Voice Recognition toggle
// (it is visually nested but functionally independent — the Enable toggle
// does not gate it). Configuration here uses a LOCAL DRAFT: edits stay in
// component state until the user clicks Save or Start Server. The rest of
// the VoiceSection keeps autoSave semantics; only this card is manual.
//
// Start Server = persist the draft, then POST /api/voice/server/start (the
//   asr-server auto-loads the configured model on startup).
// Stop Server  = POST /api/voice/server/stop (kills the supervised process).
function LocalAsrServerCard({
  initialServer,
  initialLoadOnStartup,
  modelStatus,
  availableDevices,
  asrReachable,
  fullConfig,
  onPersisted,
  onConfigRefreshed,
  onModelStatus,
  t,
}: {
  initialServer: AsrServerConfig;
  initialLoadOnStartup: boolean;
  modelStatus: AsrModelStatus;
  availableDevices: string[];
  asrReachable: boolean;
  /** Latest full VoiceConfig, used to build the PUT body on Save/Start. */
  fullConfig: VoiceConfig;
  /** Called with the freshly-persisted VoiceConfig so the parent can sync. */
  onPersisted: (config: VoiceConfig) => void;
  /**
   * Called after Start/Stop with a freshly-fetched VoiceConfig. This lets the
   * parent refresh `asrServerReachable` + `modelStatus` — which the supervisor
   * recomputes server-side — so the title status icon and Start/Stop button
   * reflect the new process state immediately instead of waiting for the next
   * periodic poll (which only runs once the server is already reachable).
   */
  onConfigRefreshed: (config: VoiceConfig) => void;
  /** Caller-provided model-status setter so we can update the badge locally. */
  onModelStatus: (status: AsrModelStatus) => void;
  t: (key: string) => string;
}) {
  // Local draft — independent of the parent's autoSave flow.
  const [draftServer, setDraftServer] = useState<AsrServerConfig>(initialServer);
  const [draftLoadOnStartup, setDraftLoadOnStartup] = useState<boolean>(initialLoadOnStartup);
  const [expanded, setExpanded] = useState(true);
  const [busy, setBusy] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  // Re-sync the draft if the server-pushed config changes underneath us
  // (e.g. a Start/Stop round-trip returns a refreshed snapshot). We compare
  // JSON to avoid clobbering in-flight edits when the parent re-renders.
  const incomingSnapshot = useMemo(
    () => JSON.stringify({ s: initialServer, l: initialLoadOnStartup }),
    [initialServer, initialLoadOnStartup],
  );
  const lastSyncedRef = useRef(incomingSnapshot);
  useEffect(() => {
    if (incomingSnapshot !== lastSyncedRef.current) {
      lastSyncedRef.current = incomingSnapshot;
      setDraftServer(initialServer);
      setDraftLoadOnStartup(initialLoadOnStartup);
    }
  }, [incomingSnapshot, initialServer, initialLoadOnStartup]);

  const isDirty = useMemo(() => {
    const draftSnapshot = JSON.stringify({ s: draftServer, l: draftLoadOnStartup });
    return draftSnapshot !== lastSyncedRef.current;
  }, [draftServer, draftLoadOnStartup]);

// Conventional display spellings for inference backends; the server reports
// lowercase ids (cpu/cuda/...). Unknown backends fall back to the raw id.
const DEVICE_LABELS: Record<string, string> = {
  cpu: 'CPU',
  cuda: 'CUDA',
  vulkan: 'Vulkan',
  metal: 'Metal',
};

const deviceOptions = useMemo(() => {
  const opts = availableDevices.map((d) => ({
    value: d,
    label: DEVICE_LABELS[d] ?? d,
  }));
  opts.unshift({ value: 'auto', label: t('voice.deviceAuto') });
  return opts;
}, [availableDevices, t]);

  const updateServer = useCallback((patch: Partial<AsrServerConfig>) => {
    setDraftServer((prev) => ({ ...prev, ...patch }));
  }, []);

  // Build the PUT body from the live parent config + our local draft, then
  // persist. Returns true on success.
  const persistDraft = useCallback(async (): Promise<boolean> => {
    setSaveError(null);
    try {
      const body: VoiceConfig = {
        ...fullConfig,
        loadOnStartup: draftLoadOnStartup,
        server: draftServer,
      };
      const saved = await updateVoiceConfig(body);
      lastSyncedRef.current = JSON.stringify({
        s: saved.server,
        l: saved.loadOnStartup,
      });
      // Keep the draft in lockstep with what the server round-tripped. The
      // server never echoes the write-only `accessToken` back, so the field
      // falls back to the fresh `accessTokenValue` for display.
      setDraftServer(saved.server);
      setDraftLoadOnStartup(saved.loadOnStartup);
      onPersisted(saved);
      return true;
    } catch (e) {
      setSaveError(e instanceof Error ? e.message : String(e));
      return false;
    }
  }, [fullConfig, draftLoadOnStartup, draftServer, onPersisted]);

  const handleStart = useCallback(async () => {
    setBusy(true);
    try {
      // Persist first so the supervisor spawns with the values the user is
      // looking at, then issue the start command.
      const ok = await persistDraft();
      if (!ok) return;
      await startVoiceServer();
      // Refresh the parent's snapshot so `asrServerReachable` flips true and
      // the title status icon + Start/Stop button update immediately. Without
      // this, the parent's poll effect stays gated on the stale (pre-start)
      // `asrServerReachable=false` and never picks up the new state.
      const refreshed = await getVoiceConfig();
      onConfigRefreshed(refreshed);
      onModelStatus(refreshed.modelStatus);
    } catch (e) {
      setSaveError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, [persistDraft, onConfigRefreshed, onModelStatus]);

  const handleStop = useCallback(async () => {
    setBusy(true);
    setSaveError(null);
    try {
      await stopVoiceServer();
      // Refresh so asrServerReachable flips false and the button flips back
      // to "Start Server".
      const refreshed = await getVoiceConfig();
      onConfigRefreshed(refreshed);
      onModelStatus(refreshed.modelStatus);
    } catch (e) {
      setSaveError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }, [onConfigRefreshed, onModelStatus]);

  const handleSave = useCallback(() => { void persistDraft(); }, [persistDraft]);

  return (
    <CollapsibleCard
      title={t('voice.localServer')}
      expanded={expanded}
      onToggle={() => setExpanded(!expanded)}
      statusIcon={
        <LocalAsrStatusIcon
          reachable={asrReachable}
          modelStatus={modelStatus}
          t={t}
        />
      }
    >
      <div className="space-y-4">
        <FormField label={t('voice.loadOnStartup')} description={t('voice.loadOnStartupDesc')}>
          <ToggleField
            enabled={draftLoadOnStartup}
            onChange={setDraftLoadOnStartup}
          />
        </FormField>

        {/* Binding address — own row, dropdown like MCP */}
        <FormField label={t('voice.bindAddress')} description={t('voice.bindAddressDesc')}>
          <SelectField
            value={draftServer.host === '0.0.0.0' ? '0.0.0.0' : '127.0.0.1'}
            onChange={(e) => updateServer({ host: e.target.value })}
            options={[
              { value: '127.0.0.1', label: t('mcp.bindLocalhost') },
              { value: '0.0.0.0', label: t('voice.bindAnyAddress') },
            ]}
          />
        </FormField>

        {/* Port — own row */}
        <FormField label={t('voice.port')}>
          <input
            type="number"
            min={0}
            max={65535}
            value={draftServer.port}
            onChange={(e) => updateServer({ port: Number(e.target.value) || 0 })}
            className={inputClassName}
            placeholder="8398"
          />
        </FormField>

        <LocalAsrModelManager
          directory={draftServer.modelPath}
          onDirectoryChange={(modelPath) => updateServer({
            modelPath,
            // Existing configurations only stored a full model path. Once the
            // user chooses a model root, make the visible default selection
            // explicit so the next save uses the new root + relative layout.
            modelId: draftServer.modelId || LOCAL_ASR_MODELS[0].id,
            modelRelativePath: draftServer.modelRelativePath || LOCAL_ASR_MODELS[0].id,
          })}
          selectedId={draftServer.modelId || LOCAL_ASR_MODELS[0].id}
          onModelChange={(modelId, modelRelativePath) => updateServer({ modelId, modelRelativePath })}
          t={t}
        />

        <FormField label={t('voice.device')} description={t('voice.deviceDesc')}>
          <SelectField
            value={draftServer.device}
            onChange={(e) => updateServer({ device: e.target.value })}
            options={deviceOptions}
          />
        </FormField>

        {(draftServer.device === 'auto' || draftServer.device.startsWith('cuda')) && (
          <div className="flex items-start gap-2 rounded-standard border border-amber-200 bg-amber-50 px-3 py-2 text-xs text-amber-800">
            <AlertCircle className="mt-0.5 h-4 w-4 shrink-0" />
            <div className="space-y-1">
              <p>{t('voice.nvidiaDriverHint')}</p>
              <a
                className="inline-flex items-center gap-1 font-medium text-accent-text hover:underline"
                href="https://www.nvidia.com/drivers"
                target="_blank"
                rel="noreferrer"
              >
                {t('voice.nvidiaDriverDownload')}
                <ExternalLink className="h-3.5 w-3.5" />
              </a>
            </div>
          </div>
        )}

        {/* Access token — same inline-actions SecretField as the MCP section:
            dice generates, copy grabs the token without revealing it. The
            decrypted token is shipped from the server as `accessTokenValue`
            (the WebUI is vterm-server's trusted admin console); typing or
            generating writes the write-only `accessToken` (absent = leave
            unchanged, empty = clear). */}
        <FormField label={t('voice.accessToken')} description={t('voice.accessTokenDesc')}>
          <SecretField
            value={draftServer.accessToken ?? draftServer.accessTokenValue ?? ''}
            onChange={(e) => updateServer({ accessToken: e.target.value })}
            placeholder={t('voice.accessTokenEmpty')}
            copyText={draftServer.accessToken ?? draftServer.accessTokenValue ?? ''}
            copyTitle={t('voice.accessTokenCopy')}
            onGenerate={() => updateServer({ accessToken: generateRandomToken() })}
            generateTitle={t('voice.generateToken')}
          />
        </FormField>

        {/* Status + Save + Start/Stop */}
        <div className="flex flex-wrap items-center gap-3 pt-2">
          <ModelStatusBadge status={modelStatus} t={t} />

          <div className="flex-1" />

          {/* Manual Save — applies config changes (may trigger supervisor restart) */}
          <button
            type="button"
            onClick={handleSave}
            disabled={!isDirty || busy}
            className="px-3 py-1.5 text-xs font-medium rounded-standard bg-accent-subtle text-accent-text hover:bg-accent-subtle/80 disabled:opacity-50 disabled:cursor-not-allowed transition-colors duration-vt-base"
          >
            {busy ? t('common.saving') : isDirty ? t('voice.save') : t('voice.saved')}
          </button>

          {/* Start / Stop the supervised asr-server process */}
          <button
            type="button"
            onClick={asrReachable ? handleStop : handleStart}
            disabled={busy}
            className={`px-3 py-1.5 text-xs font-medium rounded-standard transition-colors duration-vt-base disabled:opacity-50 disabled:cursor-not-allowed ${
              asrReachable
                ? 'bg-red-50 text-red-700 hover:bg-red-100'
                : 'bg-green-50 text-green-700 hover:bg-green-100'
            }`}
          >
            {busy
              ? t('common.saving')
              : asrReachable
              ? t('voice.stopServer')
              : t('voice.startServer')}
          </button>
        </div>

        {saveError && (
          <div className="text-xs text-vt-error">{saveError}</div>
        )}
      </div>
    </CollapsibleCard>
  );
}

// --- Text Polish Service Card ---
// Compact editor for the `llm.voicePolish` LLM provider — the service that
// polishes ASR transcripts (see vterm-server/src/voice/polish.rs, which uses
// the same config for the native voice dialog). Auto-saves via /api/llm with
// the same semantics as the LLM section; `onReadyChange` reports the derived
// "enabled + provider selected" readiness so the parent can gate the Polish
// buttons in the test panel (mirrors the server's `polishEnabled` flag).
function PolishServiceCard({
  llmAddons,
  ready,
  onReadyChange,
  t,
}: {
  llmAddons: AddonInfo[];
  ready: boolean;
  onReadyChange: (ready: boolean) => void;
  t: (key: string) => string;
}) {
  const fetchFn = useCallback(() => getLlmConfig(), []);
  const saveFn = useCallback((config: LlmConfig) => updateLlmConfig(config), []);
  const { data, loading, saveError, updateData } = useConfigSection<LlmConfig>(fetchFn, saveFn);
  const [expanded, setExpanded] = useState(false);

  const enabledLlmAddons = useMemo(
    () => llmAddons.filter((a) => a.enabled),
    [llmAddons],
  );
  const providerOptions = useMemo(
    () => enabledLlmAddons.length === 0
      ? [{ value: '', label: t('llm.noProviders') }]
      : enabledLlmAddons.map((a) => ({ value: a.id, label: a.name })),
    [enabledLlmAddons, t],
  );

  const polish = data?.voicePolish;
  const selectedAddon = useMemo(
    () => enabledLlmAddons.find((a) => a.id === polish?.provider) ?? null,
    [enabledLlmAddons, polish?.provider],
  );
  const resolvedConfig = useMemo(
    () => (selectedAddon && polish ? resolveAddonConfig(selectedAddon, polish.config) : {}),
    [selectedAddon, polish],
  );

  useEffect(() => {
    if (!polish) return;
    onReadyChange(polish.enabled && !!polish.provider?.trim());
  }, [polish, onReadyChange]);

  const patchPolish = useCallback((patch: Partial<LlmProviderConfig>) => {
    if (!data) return;
    updateData({ voicePolish: { ...data.voicePolish, ...patch } });
  }, [data, updateData]);

  return (
    <CollapsibleCard
      title={t('voice.polishService')}
      expanded={expanded}
      onToggle={() => setExpanded(!expanded)}
      statusIcon={
        <span
          className={`inline-flex items-center ${ready ? 'text-green-600' : 'text-gray-400'}`}
          title={ready ? t('voice.polishReady') : t('voice.polishNotConfigured')}
        >
          {ready ? <CheckCircle className="w-4 h-4" /> : <Circle className="w-4 h-4" />}
        </span>
      }
    >
      {loading || !polish ? (
        <div className="flex items-center justify-center py-4">
          <Loader2 className="w-4 h-4 animate-spin text-warm-black" />
        </div>
      ) : (
        <div className="space-y-4">
          <FormField label={t('voice.polishEnable')} description={t('voice.polishServiceDesc')}>
            <ToggleField
              enabled={polish.enabled}
              onChange={(enabled) => patchPolish({ enabled })}
            />
          </FormField>

          <FormField label={t('voice.polishProvider')} description={t('voice.polishProviderDesc')}>
            <SelectField
              value={providerOptions.some((o) => o.value === polish.provider) ? polish.provider : ''}
              onChange={(e) => patchPolish({ provider: e.target.value })}
              options={providerOptions}
            />
          </FormField>

          {selectedAddon && selectedAddon.configFields.length > 0 && (
            <div className="space-y-4 pt-2 border-t border-golden-sand/60">
              {selectedAddon.configFields.map((field) => (
                <FormField key={field.key} label={field.label} description={field.description}>
                  {renderAddonConfigField(
                    field,
                    resolvedConfig[field.key] ?? '',
                    (value) => patchPolish({ config: { ...polish.config, [field.key]: value } }),
                    false,
                  )}
                </FormField>
              ))}
            </div>
          )}

          {saveError && <div className="text-xs text-vt-error">{saveError}</div>}
        </div>
      )}
    </CollapsibleCard>
  );
}

// --- Record Tab ---
function WaveformVisualizer({ analyser }: { analyser: AnalyserNode | null }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animRef = useRef<number>(0);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const resize = () => {
      const rect = canvas.getBoundingClientRect();
      canvas.width = rect.width * devicePixelRatio;
      canvas.height = rect.height * devicePixelRatio;
      ctx.setTransform(devicePixelRatio, 0, 0, devicePixelRatio, 0, 0);
    };
    resize();

    const draw = () => {
      const w = canvas.getBoundingClientRect().width;
      const h = canvas.getBoundingClientRect().height;

      ctx.fillStyle = '#0c0c20';
      ctx.fillRect(0, 0, w, h);

      if (!analyser) {
        ctx.strokeStyle = '#1e1e3a';
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(0, h / 2);
        ctx.lineTo(w, h / 2);
        ctx.stroke();
        animRef.current = requestAnimationFrame(draw);
        return;
      }

      const bufLen = analyser.frequencyBinCount;
      const dataArr = new Uint8Array(bufLen);
      analyser.getByteTimeDomainData(dataArr);

      ctx.shadowColor = '#4ecdc4';
      ctx.shadowBlur = 8;
      ctx.strokeStyle = '#4ecdc4';
      ctx.lineWidth = 2;
      ctx.beginPath();

      const sliceWidth = w / bufLen;
      let x = 0;
      for (let i = 0; i < bufLen; i++) {
        const v = (dataArr[i] ?? 128) / 128.0;
        const y = (v * h) / 2;
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
        x += sliceWidth;
      }
      ctx.lineTo(w, h / 2);
      ctx.stroke();
      ctx.shadowBlur = 0;

      animRef.current = requestAnimationFrame(draw);
    };

    animRef.current = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(animRef.current);
  }, [analyser]);

  return (
    <canvas
      ref={canvasRef}
      className="w-full h-16 rounded-standard border border-golden-sand"
    />
  );
}

function encodeWav(samples: Float32Array, sampleRate: number): Blob {
  const numSamples = samples.length;
  const dataLength = numSamples * 2;
  const buffer = new ArrayBuffer(44 + dataLength);
  const view = new DataView(buffer);

  const writeStr = (offset: number, str: string) => {
    for (let i = 0; i < str.length; i++) view.setUint8(offset + i, str.charCodeAt(i));
  };

  writeStr(0, 'RIFF');
  view.setUint32(4, 36 + dataLength, true);
  writeStr(8, 'WAVE');
  writeStr(12, 'fmt ');
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  writeStr(36, 'data');
  view.setUint32(40, dataLength, true);

  let offset = 44;
  for (let i = 0; i < numSamples; i++) {
    const s = Math.max(-1, Math.min(1, samples[i]));
    view.setInt16(offset, s < 0 ? s * 0x8000 : s * 0x7FFF, true);
    offset += 2;
  }

  return new Blob([buffer], { type: 'audio/wav' });
}

async function resampleTo16kMono(audioBuffer: AudioBuffer): Promise<Float32Array> {
  const targetRate = 16000;
  const numFrames = Math.ceil(audioBuffer.duration * targetRate);
  const offlineCtx = new OfflineAudioContext(1, numFrames, targetRate);
  const source = offlineCtx.createBufferSource();
  source.buffer = audioBuffer;
  source.connect(offlineCtx.destination);
  source.start(0);
  const rendered = await offlineCtx.startRendering();
  return rendered.getChannelData(0);
}

/// Format a millisecond duration as `mm:ss` (e.g. `00:53`).
function formatMmss(ms: number): string {
  const totalSec = Math.floor(ms / 1000);
  const mm = Math.floor(totalSec / 60);
  const ss = totalSec % 60;
  return `${String(mm).padStart(2, '0')}:${String(ss).padStart(2, '0')}`;
}

function RecordTab({ t, polishEnabled }: { t: (key: string) => string; polishEnabled: boolean }) {
  const [isRecording, setIsRecording] = useState(false);
  const [result, setResult] = useState('');
  const [error, setError] = useState('');
  const [isTranscribing, setIsTranscribing] = useState(false);
  const [transcribeMs, setTranscribeMs] = useState<number | null>(null);
  const [analyser, setAnalyser] = useState<AnalyserNode | null>(null);
  // One-click polish of the transcript (POST /api/voice/polish → the server's
  // voice_polish LLM pipeline, same prompts as the native voice dialog).
  const [isPolishing, setIsPolishing] = useState(false);
  const [polishError, setPolishError] = useState('');
  // Elapsed recording time in ms, driven by an interval while recording.
  const [recordingElapsedMs, setRecordingElapsedMs] = useState(0);
  // Total duration of the last captured audio clip (seconds), shown after stop.
  const [audioDurationSec, setAudioDurationSec] = useState<number | null>(null);
  const audioCtxRef = useRef<AudioContext | null>(null);
  const streamRef = useRef<MediaStream | null>(null);
  const recorderRef = useRef<MediaRecorder | null>(null);
  const chunksRef = useRef<Blob[]>([]);
  // Wall-clock timestamp captured when recording starts; used to drive the
  // elapsed timer and to compute the final audio duration on stop.
  const recordStartRef = useRef<number>(0);
  const elapsedTimerRef = useRef<number | null>(null);

  const cleanup = useCallback(() => {
    streamRef.current?.getTracks().forEach((track) => track.stop());
    streamRef.current = null;
    audioCtxRef.current?.close();
    audioCtxRef.current = null;
    recorderRef.current = null;
    setAnalyser(null);
    if (elapsedTimerRef.current !== null) {
      clearInterval(elapsedTimerRef.current);
      elapsedTimerRef.current = null;
    }
  }, []);

  const startRecording = useCallback(async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: { channelCount: 1, echoCancellation: false, noiseSuppression: false, autoGainControl: false },
      });
      streamRef.current = stream;

      // AudioContext at native sample rate for visualization
      const audioCtx = new AudioContext();
      audioCtxRef.current = audioCtx;
      const source = audioCtx.createMediaStreamSource(stream);

      const analyserNode = audioCtx.createAnalyser();
      analyserNode.fftSize = 2048;
      source.connect(analyserNode);
      setAnalyser(analyserNode);

      // MediaRecorder for capture (reliable, high quality)
      const recorder = new MediaRecorder(stream);
      recorderRef.current = recorder;
      chunksRef.current = [];
      recorder.ondataavailable = (e) => {
        if (e.data.size > 0) chunksRef.current.push(e.data);
      };
      recorder.start(200);

      // Start the elapsed timer (updates ~4x/sec for a smooth mm:ss display).
      recordStartRef.current = Date.now();
      setRecordingElapsedMs(0);
      elapsedTimerRef.current = window.setInterval(() => {
        setRecordingElapsedMs(Date.now() - recordStartRef.current);
      }, 250);

      setIsRecording(true);
      setResult('');
      setAudioDurationSec(null);
      setError('');
      setPolishError('');
    } catch (e) {
      cleanup();
      setError(e instanceof Error ? e.message : String(e));
    }
  }, [cleanup]);

  const stopRecording = useCallback(() => {
    const recorder = recorderRef.current;
    if (!recorder || recorder.state === 'inactive') {
      cleanup();
      setIsRecording(false);
      return;
    }

    recorder.onstop = async () => {
      const blob = new Blob(chunksRef.current, { type: recorder.mimeType });
      chunksRef.current = [];
      cleanup();
      setIsRecording(false);

      if (blob.size === 0) {
        setError('No audio captured');
        return;
      }

      try {
        // Decode browser's native format → AudioBuffer → resample to 16kHz mono → WAV
        const arrayBuf = await blob.arrayBuffer();
        const audioCtx = new AudioContext();
        const audioBuffer = await audioCtx.decodeAudioData(arrayBuf);
        const mono16k = await resampleTo16kMono(audioBuffer);
        audioCtx.close();

        const audioDurationSec = mono16k.length / 16000;
        setAudioDurationSec(audioDurationSec);

        const wavBlob = encodeWav(mono16k, 16000);

        setIsTranscribing(true);
        setError('');
        setTranscribeMs(null);
        // The asr-server's OpenAI-compatible endpoint returns only {text}
        // (no transcribe_ms), so the latency is measured client-side.
        const startedAt = performance.now();
        transcribeFile(new File([wavBlob], 'recording.wav', { type: 'audio/wav' }))
          .then((resp) => {
            setResult(resp.text);
            setTranscribeMs(performance.now() - startedAt);
          })
          .catch((e) => setError(e instanceof Error ? e.message : String(e)))
          .finally(() => setIsTranscribing(false));
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    };

    recorder.stop();
  }, [cleanup]);

  const handlePolish = useCallback(async () => {
    const text = result.trim();
    if (!text || isPolishing) return;
    setIsPolishing(true);
    setPolishError('');
    try {
      const resp = await polishVoiceText(text);
      if (resp.text) setResult(resp.text);
    } catch (e) {
      setPolishError(e instanceof Error ? e.message : String(e));
    } finally {
      setIsPolishing(false);
    }
  }, [result, isPolishing]);

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        {!isRecording ? (
          <button
            type="button"
            onClick={startRecording}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-standard bg-accent-subtle text-accent-text hover:bg-accent-subtle/80 transition-colors duration-vt-base"
          >
            <Mic className="w-4 h-4" />
            {t('voice.startRecording')}
          </button>
        ) : (
          <button
            type="button"
            onClick={stopRecording}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-standard bg-red-50 text-red-700 hover:bg-red-100 transition-colors duration-vt-base"
          >
            <Square className="w-4 h-4" />
            {t('voice.stopRecording')}
          </button>
        )}
        {isRecording && (
          <span className="flex items-center gap-1.5 text-xs text-red-600">
            <span className="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
            {t('voice.recording')}
            <span className="font-mono ml-1">{formatMmss(recordingElapsedMs)}</span>
          </span>
        )}
      </div>
      <WaveformVisualizer analyser={analyser} />
      {isTranscribing && (
        <div className="flex items-center gap-2 text-xs text-warm-khaki">
          <Loader2 className="w-3.5 h-3.5 animate-spin" />
          {t('voice.transcribing')}
        </div>
      )}
      {error && (
        <div className="text-xs text-vt-error">{t('voice.transcriptionFailed').replace('{error}', error)}</div>
      )}
      {result && (
        <div className="space-y-2">
          <div className="flex items-center justify-between gap-2">
            <span className="text-xs font-semibold text-warm-black">{t('voice.transcriptionResult')}</span>
            <div className="flex items-center gap-1">
              {/* Mic toggle — start/stop dictation straight from the text box. */}
              <button
                type="button"
                onClick={isRecording ? stopRecording : startRecording}
                className={`p-1.5 rounded-standard transition-colors duration-vt-base ${
                  isRecording
                    ? 'text-red-600 hover:bg-red-50'
                    : 'text-deep-olive hover:text-warm-black hover:bg-golden-cream'
                }`}
                title={isRecording ? t('voice.stopRecording') : t('voice.startRecording')}
                aria-label={isRecording ? t('voice.stopRecording') : t('voice.startRecording')}
              >
                {isRecording ? <Square className="w-3.5 h-3.5" /> : <Mic className="w-3.5 h-3.5" />}
              </button>
              {/* One-click polish of the transcript via the voice_polish LLM. */}
              <button
                type="button"
                onClick={() => { void handlePolish(); }}
                disabled={!polishEnabled || isPolishing || !result.trim()}
                className={`p-1.5 rounded-standard transition-colors duration-vt-base disabled:opacity-40 disabled:cursor-not-allowed ${
                  isPolishing ? 'text-amber-600' : 'text-deep-olive hover:text-warm-black hover:bg-golden-cream'
                }`}
                title={!polishEnabled ? t('voice.polishUnavailable') : isPolishing ? t('voice.polishing') : t('voice.polish')}
                aria-label={t('voice.polish')}
              >
                <Sparkles className={`w-3.5 h-3.5 ${isPolishing ? 'animate-pulse' : ''}`} />
              </button>
              <CopyButton text={result} title={t('voice.copyResult')} />
            </div>
          </div>
          <textarea
            value={result}
            onChange={(e) => setResult(e.target.value)}
            className={`${inputClassName} min-h-[80px]`}
          />
          {isPolishing && (
            <div className="flex items-center gap-2 text-xs text-warm-khaki">
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              {t('voice.polishing')}
            </div>
          )}
          {polishError && (
            <div className="text-xs text-vt-error">{t('voice.polishFailed').replace('{error}', polishError)}</div>
          )}
          {transcribeMs !== null && (
            <p className="text-xs text-deep-olive font-mono">
              {audioDurationSec !== null ? `Audio ${audioDurationSec.toFixed(2)}s · ` : ''}Transcribed in {(transcribeMs / 1000).toFixed(2)}s ({transcribeMs.toFixed(1)}ms)
            </p>
          )}
        </div>
      )}
    </div>
  );
}

// --- Live Tab ---
function LiveTab({ t, polishEnabled }: { t: (key: string) => string; polishEnabled: boolean }) {
  const {
    committed, unstable, isRecording, isConnecting, error, analyserNode, start, stop, setCommitted,
  } = useLiveTranscriber();
  const [isPolishing, setIsPolishing] = useState(false);
  const [polishError, setPolishError] = useState('');

  const handlePolish = useCallback(async () => {
    const text = committed.trim();
    if (!text || isPolishing) return;
    setIsPolishing(true);
    setPolishError('');
    try {
      const resp = await polishVoiceText(text);
      if (resp.text) setCommitted(resp.text);
    } catch (e) {
      setPolishError(e instanceof Error ? e.message : String(e));
    } finally {
      setIsPolishing(false);
    }
  }, [committed, isPolishing, setCommitted]);

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        {!isRecording && !isConnecting ? (
          <button
            type="button"
            onClick={start}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-standard bg-accent-subtle text-accent-text hover:bg-accent-subtle/80 transition-colors duration-vt-base"
          >
            <Radio className="w-4 h-4" />
            {t('voice.startListening')}
          </button>
        ) : (
          <button
            type="button"
            onClick={stop}
            disabled={isConnecting}
            className="flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-standard bg-red-50 text-red-700 hover:bg-red-100 transition-colors duration-vt-base disabled:opacity-50"
          >
            {isConnecting ? <Loader2 className="w-4 h-4 animate-spin" /> : <Square className="w-4 h-4" />}
            {isConnecting ? t('voice.connecting') : t('voice.stopListening')}
          </button>
        )}
        {isRecording && (
          <span className="flex items-center gap-1.5 text-xs text-red-600">
            <span className="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
            {t('voice.listening')}
          </span>
        )}
      </div>
      <WaveformVisualizer analyser={analyserNode} />
      {error && (
        <div className="text-xs text-vt-error">{t('voice.transcriptionFailed').replace('{error}', error)}</div>
      )}
      {(committed || unstable) && (
        <div className="space-y-2">
          <div className="flex items-center justify-between gap-2">
            <span className="text-xs font-semibold text-warm-black">{t('voice.transcriptionResult')}</span>
            <div className="flex items-center gap-1">
              {/* Mic toggle — pause/resume dictation from the text box. */}
              <button
                type="button"
                onClick={isRecording ? stop : start}
                disabled={isConnecting}
                className={`p-1.5 rounded-standard transition-colors duration-vt-base disabled:opacity-50 ${
                  isRecording
                    ? 'text-red-600 hover:bg-red-50'
                    : 'text-deep-olive hover:text-warm-black hover:bg-golden-cream'
                }`}
                title={isRecording ? t('voice.stopListening') : t('voice.startListening')}
                aria-label={isRecording ? t('voice.stopListening') : t('voice.startListening')}
              >
                {isRecording ? <Square className="w-3.5 h-3.5" /> : <Mic className="w-3.5 h-3.5" />}
              </button>
              {/* One-click polish of the committed transcript. */}
              <button
                type="button"
                onClick={() => { void handlePolish(); }}
                disabled={!polishEnabled || isPolishing || isRecording || !committed.trim()}
                className={`p-1.5 rounded-standard transition-colors duration-vt-base disabled:opacity-40 disabled:cursor-not-allowed ${
                  isPolishing ? 'text-amber-600' : 'text-deep-olive hover:text-warm-black hover:bg-golden-cream'
                }`}
                title={!polishEnabled ? t('voice.polishUnavailable') : isPolishing ? t('voice.polishing') : t('voice.polish')}
                aria-label={t('voice.polish')}
              >
                <Sparkles className={`w-3.5 h-3.5 ${isPolishing ? 'animate-pulse' : ''}`} />
              </button>
              <CopyButton text={committed} title={t('voice.copyResult')} />
            </div>
          </div>
          <div className="w-full bg-bg-inset px-2 py-1.5 text-sm font-normal rounded-standard border border-golden-sand min-h-[80px] whitespace-pre-wrap break-words">
            <span className="text-warm-black">{committed}</span>
            {unstable && <span style={{ color: 'rgba(0,0,0,0.35)' }}>{unstable}</span>}
          </div>
          {isPolishing && (
            <div className="flex items-center gap-2 text-xs text-warm-khaki">
              <Loader2 className="w-3.5 h-3.5 animate-spin" />
              {t('voice.polishing')}
            </div>
          )}
          {polishError && (
            <div className="text-xs text-vt-error">{t('voice.polishFailed').replace('{error}', polishError)}</div>
          )}
        </div>
      )}
    </div>
  );
}

// --- Tab type ---
type TestTab = 'record' | 'live';

// --- Main VoiceSection ---
export function VoiceSection() {
  const { t } = useTranslation();
  const [asrAddons, setAsrAddons] = useState<AddonInfo[]>([]);
  const [llmAddons, setLlmAddons] = useState<AddonInfo[]>([]);
  const [providerExpanded, setProviderExpanded] = useState(false);
  const [testExpanded, setTestExpanded] = useState(false);
  const [testTab, setTestTab] = useState<TestTab>('record');
  const [modelStatus, setModelStatus] = useState<AsrModelStatus | null>(null);
  // Local override for the polish-service readiness, reported by
  // PolishServiceCard as the user edits the LLM config. Falls back to the
  // `polishEnabled` flag computed server-side on GET /api/voice.
  const [polishReady, setPolishReady] = useState<boolean | null>(null);
  const handlePolishReadyChange = useCallback((ready: boolean) => setPolishReady(ready), []);

  const fetchFn = useCallback(() => getVoiceConfig(), []);
  const saveFn = useCallback((config: VoiceConfig) => updateVoiceConfig(config), []);
  // autoSave=true (the default) — Provider config and the Enable toggle save
  // on change. The Local ASR Server card opts out: it keeps
  // its own local draft and persists via its Save / Start Server buttons.
  const { data, loading, saving, loadError, saveError, updateData, updateField } =
    useConfigSection<VoiceConfig>(fetchFn, saveFn);

  useEffect(() => {
    let cancelled = false;
    getAddons()
      .then((addons) => {
        if (!cancelled) {
          setAsrAddons(addons.filter((a) => a.addonType === 'asr' || a.addonType === 'stt'));
          setLlmAddons(addons.filter((a) => a.addonType === 'llm'));
        }
      })
      .catch(() => {});
    return () => { cancelled = true; };
  }, []);

  // Poll model status
  useEffect(() => {
    if (!data?.asrServerReachable) return;
    let cancelled = false;
    const poll = async () => {
      try {
        const status = await getVoiceModelStatus();
        if (!cancelled) setModelStatus(status);
      } catch { /* ignore */ }
    };
    poll();
    const intervalMs = modelStatus?.status === 'loading' ? 2000 : 5000;
    const interval = setInterval(poll, intervalMs);
    return () => { cancelled = true; clearInterval(interval); };
  }, [data?.asrServerReachable, modelStatus?.status]);

  const enabledAsrAddons = useMemo(
    () => asrAddons.filter((a) => a.enabled),
    [asrAddons],
  );

  const providerOptions = useMemo(() => {
    // Only ASR addons appear here — there's no separate "Local ASR Server"
    // provider entry. The supervised vterm-voice process is reached via the
    // `vibe-voice` builtin addon (whose base_url defaults to the supervised
    // endpoint), so selecting that addon IS the way to use the local server.
    return enabledAsrAddons.map((a) => ({ value: a.id, label: a.name }));
  }, [enabledAsrAddons]);

  const selectedAsrAddon = useMemo(
    () => enabledAsrAddons.find((a) => a.id === data?.provider) ?? null,
    [enabledAsrAddons, data?.provider],
  );

  const resolvedProviderConfig = useMemo(
    () => selectedAsrAddon ? resolveAddonConfig(selectedAsrAddon, data?.providerConfig ?? {}) : {},
    [selectedAsrAddon, data?.providerConfig],
  );

  const handleProviderConfigChange = useCallback(
    (key: string, value: string) => {
      if (!data) return;
      updateData({
        providerConfig: { ...data.providerConfig, [key]: value },
      });
    },
    [data, updateData],
  );

  // One-click: copy the local ASR server's address + access token into the
  // vibe-voice provider fields. Always reaches the supervised server over
  // loopback (127.0.0.1) regardless of the configured bind address, matching
  // the addon's default base URL. The token is only written when one is set
  // so an unconfigured server doesn't clobber an existing remote token.
  const handleFillFromLocalAsr = useCallback(() => {
    if (!data) return;
    const token = data.server.accessTokenValue;
    updateData({
      providerConfig: {
        ...data.providerConfig,
        base_url: `https://127.0.0.1:${data.server.port}`,
        ...(token ? { access_token: token } : {}),
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
        <span className="font-light">{loadError ?? t('voice.loadError')}</span>
      </div>
    );
  }

  const currentModelStatus = modelStatus ?? data.modelStatus ?? { status: 'not_loaded' as const };
  // Polish readiness: live value from the Polish service card once it has
  // loaded, otherwise the server-computed flag from GET /api/voice.
  const polishEnabled = polishReady ?? data.polishEnabled === true;

  return (
    <div className="space-y-6">
      <SectionHeader title={t('voice.title')} description={t('voice.description')} />

      <div className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        {!data.asrServerReachable && selectedAsrAddon && (
          <div className="flex items-center gap-2 text-xs text-amber-700 bg-amber-50 border border-amber-200 rounded-standard px-3 py-2">
            <AlertCircle className="w-4 h-4 shrink-0" />
            {t('voice.asrServerNotReachable')}
          </div>
        )}

        {/* Enable toggle */}
        <FormField label={t('voice.enable')} description={t('voice.enableDesc')}>
          <ToggleField
            enabled={data.enabled}
            onChange={(v) => updateField('enabled', v)}
          />
        </FormField>

        {/* Local ASR Server — nested under Enable visually, but functionally
            independent. Its own Enable (Start/Stop) is separate from the
            voice.enabled toggle above; either can be on without the other. */}
        <div className="pl-0 md:pl-2 border-l-0 md:border-l-2 md:border-golden-sand/60 md:ml-2">
          <LocalAsrServerCard
            initialServer={data.server}
            initialLoadOnStartup={data.loadOnStartup}
            modelStatus={currentModelStatus}
            availableDevices={data.availableDevices}
            asrReachable={data.asrServerReachable}
            fullConfig={data}
            onPersisted={(saved) => {
              // Sync the parent's snapshot so polling / other cards see the
              // freshly-saved config. updateData merges the server field.
              updateData({
                server: saved.server,
                loadOnStartup: saved.loadOnStartup,
              });
            }}
            onConfigRefreshed={(refreshed) => {
              // After Start/Stop, replace the parent snapshot wholesale so
              // `asrServerReachable` (which the supervisor recomputes) flips.
              updateData(refreshed);
            }}
            onModelStatus={(status) => setModelStatus(status)}
            t={t}
          />
        </div>
      </div>

      {/* Provider Configuration (collapsible) */}
      <CollapsibleCard
        title={t('voice.providerConfig')}
        expanded={providerExpanded}
        onToggle={() => setProviderExpanded(!providerExpanded)}
        statusIcon={
          selectedAsrAddon ? (
            <LocalAsrStatusIcon
              reachable={data.asrServerReachable}
              modelStatus={currentModelStatus}
              t={t}
            />
          ) : undefined
        }
      >
        <div className="space-y-4">
          {/* Provider dropdown — ASR addons only. No "Local ASR Server" entry:
              the supervised vterm-voice is reached via the vibe-voice addon. */}
          <FormField label={t('voice.provider')} description={t('voice.providerDesc')}>
            <SelectField
              value={data.provider}
              onChange={(e) => updateField('provider', e.target.value)}
              options={providerOptions}
            />
          </FormField>

          {/* Dynamic addon config fields — shown for the selected addon. This
              includes per-provider language selection (the vibe-voice addon
              declares a `language` field). */}
          {selectedAsrAddon && selectedAsrAddon.configFields.length > 0 && (
            <div className="space-y-4 pt-2 border-t border-golden-sand/60">
              {selectedAsrAddon.id === 'vibe-voice' && data.server.port > 0 && (
                <div className="flex items-center justify-between gap-3 rounded-standard bg-bg-inset/50 border border-golden-sand/60 px-3 py-2">
                  <span className="text-xs text-deep-olive">{t('voice.fillFromLocalAsrDesc')}</span>
                  <button
                    type="button"
                    onClick={handleFillFromLocalAsr}
                    className="inline-flex items-center gap-1.5 px-2.5 py-1.5 text-xs font-medium rounded-standard bg-accent-subtle text-accent-text hover:bg-accent-subtle/80 transition-colors duration-vt-base shrink-0"
                  >
                    <Zap className="w-3.5 h-3.5" />
                    {t('voice.fillFromLocalAsr')}
                  </button>
                </div>
              )}
              {selectedAsrAddon.configFields.map((field) => (
                <FormField key={field.key} label={field.label} description={field.description}>
                  {renderAddonConfigField(
                    field,
                    resolvedProviderConfig[field.key] ?? '',
                    (value) => handleProviderConfigChange(field.key, value),
                    false,
                  )}
                </FormField>
              ))}
            </div>
          )}
          {(!selectedAsrAddon || selectedAsrAddon.configFields.length === 0) && (
            <p className="text-xs text-deep-olive">{t('voice.noProviders')}</p>
          )}
        </div>
      </CollapsibleCard>

      {/* Text Polish Service (collapsible) — the LLM behind the Polish button
          in the test panel (and the native voice dialog). */}
      <PolishServiceCard
        llmAddons={llmAddons}
        ready={polishEnabled}
        onReadyChange={handlePolishReadyChange}
        t={t}
      />

      {/* Test Panel (collapsible) */}
      <CollapsibleCard
        title={t('voice.test')}
        expanded={testExpanded}
        onToggle={() => setTestExpanded(!testExpanded)}
      >
        <div className="space-y-4">
          <div className="flex gap-1 border-b border-golden-sand/60">
            {(['record', 'live'] as TestTab[]).map((tab) => (
              <button
                key={tab}
                type="button"
                onClick={() => setTestTab(tab)}
                className={`px-3 py-1.5 text-xs font-medium transition-colors duration-vt-base ${
                  testTab === tab
                    ? 'text-warm-black border-b-2 border-amber-gold'
                    : 'text-deep-olive hover:text-warm-black'
                }`}
              >
                {t(`voice.test${tab.charAt(0).toUpperCase() + tab.slice(1)}`)}
              </button>
            ))}
          </div>
          {testTab === 'record' && <RecordTab t={t} polishEnabled={polishEnabled} />}
          {testTab === 'live' && <LiveTab t={t} polishEnabled={polishEnabled} />}
        </div>
      </CollapsibleCard>

      {(saving || saveError) && (
        <div className={`text-xs ${saveError ? 'text-vt-error' : 'text-warm-khaki'}`}>
          {saveError ?? t('common.saving')}
        </div>
      )}
    </div>
  );
}
