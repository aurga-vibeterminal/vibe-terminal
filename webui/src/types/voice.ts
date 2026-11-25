export interface AsrServerConfig {
  port: number;
  host: string;
  /** Root directory containing locally managed ASR models. */
  modelPath: string;
  /** Identifier of the selected model in the local ASR catalog. */
  modelId: string;
  /** Path to the selected model relative to modelPath. */
  modelRelativePath: string;
  device: string;
  /**
   * Local-only, write-only: plaintext token sent by the client to *set* the
   * access token (replaces the current value; empty string clears it; absent
   * leaves it unchanged). The server does not echo this field back.
   */
  accessToken?: string;
  /**
   * The decrypted current access token, surfaced so the UI's Copy-to-clipboard
   * affordance can hand it to the user. The WebUI is the trusted admin console
   * for vterm-server. Undefined means no token is configured.
   */
  accessTokenValue?: string;
}

export type AsrModelStatus =
  | { status: 'not_loaded' }
  | { status: 'loading' }
  | { status: 'loaded'; modelId: string; device: string }
  | { status: 'error'; message: string };

/** A locally managed ASR model discovered under the configured model directory. */
export interface LocalAsrModelAvailability {
  id: string;
  available: boolean;
  path?: string;
  download?: {
    status: 'queued' | 'downloading' | 'completed' | 'failed';
    progress?: number;
    message?: string;
  };
}

export type LocalAsrModelSource = 'huggingface' | 'modelscope';

export interface LocalAsrModelScan {
  models: LocalAsrModelAvailability[];
}

export interface VoiceConfig {
  enabled: boolean;
  loadOnStartup: boolean;
  provider: string;
  providerConfig: Record<string, string>;
  server: AsrServerConfig;
  modelStatus: AsrModelStatus;
  availableDevices: string[];
  asrServerReachable: boolean;
  inputDevice?: string;
  /**
   * Whether the text-polish LLM (llm.voicePolish) is configured and enabled.
   * Gates the Polish buttons in the test panel; configure it in the Voice
   * section's Polish Service card or the LLM section.
   */
  polishEnabled?: boolean;
}
