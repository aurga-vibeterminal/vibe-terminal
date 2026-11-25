import type { ServerConfig } from '@/types';

export async function getServerConfig(): Promise<ServerConfig> {
  return requestJson<ServerConfig>('/api/server');
}

export async function updateServerConfig(config: ServerConfig): Promise<ServerConfig> {
  return requestJson<ServerConfig>('/api/server', {
    method: 'PUT',
    body: JSON.stringify(config),
  });
}

/** macOS-only: ask the server host to open System Settings on the Full Disk
    Access pane (the nested server app needs its own grant). */
export async function openFullDiskAccessSettings(): Promise<void> {
  await requestJson<{ ok: boolean }>('/api/system/open-full-disk-access', {
    method: 'POST',
    body: '{}',
  });
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
