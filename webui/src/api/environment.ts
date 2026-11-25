import type {
  CreateNodeRequest,
  EnvironmentConfig,
  EnvironmentNode,
  MoveNodeRequest,
  RenameNodeRequest,
  UpdateLeafRequest,
} from '@/types';

export function getEnvironmentConfig(): Promise<EnvironmentConfig> {
  return requestJson<EnvironmentConfig>('/api/environment');
}

export function createNode(request: CreateNodeRequest): Promise<EnvironmentNode> {
  return requestJson<EnvironmentNode>('/api/environment/nodes', {
    method: 'POST',
    body: JSON.stringify(request),
  });
}

export function renameNode(id: string, request: RenameNodeRequest): Promise<EnvironmentNode> {
  return requestJson<EnvironmentNode>(`/api/environment/nodes/${encodeURIComponent(id)}`, {
    method: 'PUT',
    body: JSON.stringify(request),
  });
}

export function updateLeafVariables(
  id: string,
  request: UpdateLeafRequest,
): Promise<EnvironmentNode> {
  return requestJson<EnvironmentNode>(
    `/api/environment/nodes/${encodeURIComponent(id)}/variables`,
    {
      method: 'PUT',
      body: JSON.stringify(request),
    },
  );
}

export async function duplicateNode(id: string): Promise<EnvironmentNode> {
  return requestJson<EnvironmentNode>(
    `/api/environment/nodes/${encodeURIComponent(id)}/duplicate`,
    {
      method: 'POST',
    },
  );
}

export async function moveNode(id: string, request: MoveNodeRequest): Promise<void> {
  const response = await fetch(
    `/api/environment/nodes/${encodeURIComponent(id)}/move`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    },
  );
  if (!response.ok && response.status !== 204) {
    throw new Error(await readErrorMessage(response));
  }
}

export async function removeNode(id: string): Promise<void> {
  const response = await fetch(`/api/environment/nodes/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  if (!response.ok && response.status !== 204) {
    throw new Error(await readErrorMessage(response));
  }
}

async function requestJson<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, {
    headers: { 'Content-Type': 'application/json', ...init?.headers },
    ...init,
  });
  if (!response.ok) {
    throw new Error(await readErrorMessage(response));
  }
  return (await response.json()) as T;
}

async function readErrorMessage(response: Response): Promise<string> {
  const text = await response.text();
  if (!text) return `Request failed with status ${response.status}`;
  try {
    const data = JSON.parse(text) as { error?: string };
    return data.error ?? text;
  } catch {
    return text;
  }
}
