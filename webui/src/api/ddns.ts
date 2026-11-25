import type { DdnsConfig, ServerNetworkInterface } from '@/types';

export async function getDdnsConfig(): Promise<DdnsConfig> {
  return requestJson<DdnsConfig>('/api/ddns');
}

export async function updateDdnsConfig(config: DdnsConfig): Promise<DdnsConfig> {
  return requestJson<DdnsConfig>('/api/ddns', {
    method: 'PUT',
    body: JSON.stringify(config),
  });
}

export async function triggerDdnsSync(): Promise<DdnsConfig> {
  await requestJson('/api/ddns/sync', { method: 'POST' });
  return getDdnsConfig();
}

export async function getDdnsNetworkInterfaces(): Promise<ServerNetworkInterface[]> {
  return requestJson<ServerNetworkInterface[]>('/api/ddns/interfaces');
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
