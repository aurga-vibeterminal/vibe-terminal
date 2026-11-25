import type { LlmConfig, LlmStatsResponse } from '@/types';

export async function getLlmConfig(): Promise<LlmConfig> {
  return requestJson<LlmConfig>('/api/llm');
}

export async function updateLlmConfig(config: LlmConfig): Promise<LlmConfig> {
  return requestJson<LlmConfig>('/api/llm', {
    method: 'PUT',
    body: JSON.stringify(config),
  });
}

export interface LlmStatsParams {
  fromMs?: number;
  toMs?: number;
  category?: string;
  addon?: string;
}

export async function getLlmStats(params?: LlmStatsParams): Promise<LlmStatsResponse> {
  const query = new URLSearchParams();
  if (params?.fromMs) query.set('fromMs', String(params.fromMs));
  if (params?.toMs) query.set('toMs', String(params.toMs));
  if (params?.category) query.set('category', params.category);
  if (params?.addon) query.set('addon', params.addon);
  const qs = query.toString();
  const url = qs ? `/api/llm/stats?${qs}` : '/api/llm/stats';
  return requestJson<LlmStatsResponse>(url);
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
