import type { TerminalConfig } from '@/types';

export async function getTerminalConfig(): Promise<TerminalConfig> {
  return requestJson<TerminalConfig>('/api/terminal');
}

export async function updateTerminalConfig(config: TerminalConfig): Promise<TerminalConfig> {
  return requestJson<TerminalConfig>('/api/terminal', {
    method: 'PUT',
    body: JSON.stringify(config),
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
