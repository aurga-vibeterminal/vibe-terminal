import type { McpConfig, McpEndpoint, McpTemplate, McpTemplateUpdate } from '@/types';

export async function getMcpConfig(): Promise<McpConfig> {
  return requestJson<McpConfig>('/api/mcp');
}

export async function getMcpEndpoints(): Promise<McpEndpoint[]> {
  return requestJson<McpEndpoint[]>('/api/mcp/endpoints');
}

export async function updateMcpConfig(config: McpConfig): Promise<McpConfig> {
  return requestJson<McpConfig>('/api/mcp', {
    method: 'PUT',
    body: JSON.stringify(config),
  });
}

export async function getMcpTemplates(): Promise<McpTemplate[]> {
  return requestJson<McpTemplate[]>('/api/mcp/templates');
}

export async function saveMcpTemplate(
  id: string,
  update: McpTemplateUpdate,
): Promise<McpTemplate> {
  return requestJson<McpTemplate>(`/api/mcp/templates/${encodeURIComponent(id)}`, {
    method: 'PUT',
    body: JSON.stringify(update),
  });
}

export async function deleteMcpTemplate(id: string): Promise<void> {
  const response = await fetch(`/api/mcp/templates/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  if (!response.ok) {
    throw new Error(await readErrorMessage(response));
  }
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
