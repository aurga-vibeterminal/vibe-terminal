import type {
  CreatePortForwardRequest,
  PortForward,
  PortForwardShare,
  PortForwardingConfig,
} from '@/types';

export async function getPortForwardingConfig(): Promise<PortForwardingConfig> {
  return requestJson<PortForwardingConfig>('/api/port-forwarding');
}

export async function createPortForward(req: CreatePortForwardRequest): Promise<PortForward> {
  return requestJson<PortForward>('/api/port-forwarding', {
    method: 'POST',
    body: JSON.stringify(req),
  });
}

export async function updatePortForwardingConfig(
  config: Pick<
    PortForwardingConfig,
    'enabled' | 'idleTimeoutMinutes' | 'portRangeStart' | 'portRangeEnd'
  >,
): Promise<PortForwardingConfig> {
  return requestJson<PortForwardingConfig>('/api/port-forwarding', {
    method: 'PUT',
    body: JSON.stringify(config),
  });
}

export async function stopPortForward(proxyPort: number): Promise<void> {
  const response = await fetch(`/api/port-forwarding/${proxyPort}/stop`, {
    method: 'POST',
  });

  if (!response.ok) {
    throw new Error(await readErrorMessage(response));
  }
}

export async function createPortForwardShare(proxyPort: number): Promise<PortForwardShare> {
  return requestJson<PortForwardShare>(`/api/port-forwarding/${proxyPort}/share`, {
    method: 'POST',
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
