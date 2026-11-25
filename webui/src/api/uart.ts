import type { UartConfig } from '@/types';

export async function getUartConfig(): Promise<UartConfig> {
  return requestJson<UartConfig>('/api/uart');
}

export async function updateUartConfig(config: UartConfig): Promise<UartConfig> {
  return requestJson<UartConfig>('/api/uart', {
    method: 'PUT',
    body: JSON.stringify(config),
  });
}

export async function detectUartSerialPorts(): Promise<string[]> {
  return requestJson<string[]>('/api/uart/serial-ports');
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
