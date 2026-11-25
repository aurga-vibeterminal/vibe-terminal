import type { RemoteDesktopConfig, RemoteDesktopPermissionResult } from '@/types';

export async function getRemoteDesktopConfig(): Promise<RemoteDesktopConfig> {
  return requestJson<RemoteDesktopConfig>('/api/remote-desktop');
}

export async function updateRemoteDesktopConfig(
  config: RemoteDesktopConfig,
): Promise<RemoteDesktopConfig> {
  return requestJson<RemoteDesktopConfig>('/api/remote-desktop', {
    method: 'PUT',
    body: JSON.stringify(config),
  });
}

export async function requestRemoteDesktopPermission(): Promise<RemoteDesktopPermissionResult> {
  return requestJson<RemoteDesktopPermissionResult>(
    '/api/remote-desktop/request-permission',
    {
      method: 'POST',
    },
  );
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
