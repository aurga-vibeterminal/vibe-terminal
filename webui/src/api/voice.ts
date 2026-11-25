import type {
  VoiceConfig,
  AsrModelStatus,
  LocalAsrModelAvailability,
  LocalAsrModelScan,
  LocalAsrModelSource,
} from '@/types';

export async function getVoiceConfig(): Promise<VoiceConfig> {
  return requestJson<VoiceConfig>('/api/voice');
}

export async function updateVoiceConfig(config: VoiceConfig): Promise<VoiceConfig> {
  return requestJson<VoiceConfig>('/api/voice', {
    method: 'PUT',
    body: JSON.stringify(config),
  });
}

export async function getVoiceModelStatus(): Promise<AsrModelStatus> {
  return requestJson<AsrModelStatus>('/api/voice/model');
}

export async function loadVoiceModel(): Promise<{ status: string }> {
  return requestJson<{ status: string }>('/api/voice/model/load', { method: 'POST' });
}

export async function unloadVoiceModel(): Promise<{ status: string }> {
  return requestJson<{ status: string }>('/api/voice/model/unload', { method: 'POST' });
}

/** Manual Start button: ensure the supervised vterm-asr-server is running. */
export async function startVoiceServer(): Promise<AsrModelStatus> {
  return requestJson<AsrModelStatus>('/api/voice/server/start', { method: 'POST' });
}

/** Manual Stop button: tear down the supervised vterm-asr-server. */
export async function stopVoiceServer(): Promise<AsrModelStatus> {
  return requestJson<AsrModelStatus>('/api/voice/server/stop', { method: 'POST' });
}

/**
 * Scan the local ASR model directory. The server owns filesystem inspection;
 * the WebUI only renders the returned availability and download state.
 */
export async function scanLocalAsrModels(directory: string): Promise<LocalAsrModelScan> {
  const query = `?directory=${encodeURIComponent(directory)}`;
  return requestJson<LocalAsrModelScan>(`/api/voice/models${query}`);
}

/** Start or retrieve the single server-managed download for one model. */
export async function downloadLocalAsrModel(
  id: string,
  directory: string,
  source: LocalAsrModelSource,
): Promise<LocalAsrModelAvailability> {
  return requestJson<LocalAsrModelAvailability>(`/api/voice/models/${encodeURIComponent(id)}/download`, {
    method: 'POST',
    body: JSON.stringify({ directory, source }),
  });
}

/** Transcribe an audio file via the batch endpoint. The asr-server's
 *  OpenAI-compatible response carries only `text` (no timing fields). */
export async function transcribeFile(
  file: File,
  language?: string,
): Promise<{ text: string }> {
  const form = new FormData();
  form.append('file', file);
  const query = language ? `?language=${encodeURIComponent(language)}` : '';
  const response = await fetch(`/api/voice/transcribe${query}`, {
    method: 'POST',
    body: form,
  });
  if (!response.ok) {
    throw new Error(await response.text());
  }
  return response.json();
}

/**
 * Polish a voice transcript with the configured `voice_polish` LLM. The
 * server reuses the same prompt pipeline as the native voice dialog
 * (`vterm-server/src/voice/polish.rs`); this is a plain LLM round-trip and
 * can take a few seconds.
 */
export async function polishVoiceText(text: string): Promise<{ text: string }> {
  return requestJson<{ text: string }>('/api/voice/polish', {
    method: 'POST',
    body: JSON.stringify({ text }),
  });
}

export function createLiveTranscriptionWs(): WebSocket {
  const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const url = `${proto}//${window.location.host}/api/voice/transcribe/live`;
  return new WebSocket(url);
}

async function requestJson<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, {
    headers: {
      'Content-Type': 'application/json',
      ...init?.headers,
    },
    ...init,
  });

  if (!response.ok) {
    throw new Error(await readErrorMessage(response));
  }

  return (await response.json()) as T;
}

async function readErrorMessage(response: Response): Promise<string> {
  const text = await response.text();
  if (!text) {
    return `Request failed with status ${response.status}`;
  }

  try {
    const data = JSON.parse(text) as { error?: string };
    return data.error ?? text;
  } catch {
    return text;
  }
}
