export interface LlmProviderConfig {
  enabled: boolean;
  provider: string;
  config: Record<string, string>;
  maxFailures: number;
}

export interface CodeAnalysisConfig extends LlmProviderConfig {
  outputLanguage: string;
}

export interface TranslationConfig extends LlmProviderConfig {
  outputLanguage: string;
}

export interface TerminalLlmConfig extends LlmProviderConfig {
  outputLanguage: string;
}

export interface GitLlmConfig extends LlmProviderConfig {
  outputLanguage: string;
}

export interface LlmConfig {
  terminal: TerminalLlmConfig;
  git: GitLlmConfig;
  gitConflict?: GitLlmConfig;
  chat: LlmProviderConfig;
  voicePolish: LlmProviderConfig;
  codeAnalysis: CodeAnalysisConfig;
  translation: TranslationConfig;
}

// --- LLM Stats types ---

export interface LlmStatsSummary {
  totalRequests: number;
  successCount: number;
  errorCount: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalTokens: number;
  avgDurationMs: number;
  estimatedCount: number;
}

export interface LlmStatsByAddon {
  addonName: string;
  requestCount: number;
  totalTokens: number;
  inputTokens: number;
  outputTokens: number;
  successCount: number;
  avgDurationMs: number;
}

export interface LlmStatsByCategory {
  category: string;
  requestCount: number;
  totalTokens: number;
  inputTokens: number;
  outputTokens: number;
  successCount: number;
  avgDurationMs: number;
}

export interface LlmStatsByModel {
  modelName: string;
  requestCount: number;
  totalTokens: number;
  inputTokens: number;
  outputTokens: number;
  successCount: number;
  avgDurationMs: number;
}

export interface LlmStatsTimeBucket {
  bucket: string;
  requestCount: number;
  totalTokens: number;
  inputTokens: number;
  outputTokens: number;
  successCount: number;
  avgDurationMs: number;
}

export interface LlmStatsResponse {
  summary: LlmStatsSummary;
  byAddon: LlmStatsByAddon[];
  byCategory: LlmStatsByCategory[];
  byModel: LlmStatsByModel[];
  timeSeries: LlmStatsTimeBucket[];
}
