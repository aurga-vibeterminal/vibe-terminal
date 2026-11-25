import type { DeviceConfig } from '@/types';

export async function getDeviceConfig(): Promise<DeviceConfig> {
  return requestJson<DeviceConfig>('/api/device');
}

export async function updateDeviceConfig(config: DeviceConfig): Promise<DeviceConfig> {
  return requestJson<DeviceConfig>('/api/device', {
    method: 'PUT',
    body: JSON.stringify(config),
  });
}

export async function detectSerialPorts(): Promise<string[]> {
  return requestJson<string[]>('/api/device/serial-ports');
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
