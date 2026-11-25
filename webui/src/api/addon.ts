import type { AddonDetail, AddonInfo, AddonConfigUpdate } from '@/types';

export async function getAddons(): Promise<AddonInfo[]> {
  return requestJson<AddonInfo[]>('/api/addons');
}

export async function getAddonDetail(id: string): Promise<AddonDetail> {
  return requestJson<AddonDetail>(`/api/addons/${encodeURIComponent(id)}`);
}

export async function updateAddonConfig(id: string, config: AddonConfigUpdate): Promise<AddonInfo> {
  return requestJson<AddonInfo>(`/api/addons/${encodeURIComponent(id)}/config`, {
    method: 'PUT',
    body: JSON.stringify(config),
  });
}

export async function getAddonSource(id: string): Promise<{ source: string }> {
  return requestJson<{ source: string }>(`/api/addons/${encodeURIComponent(id)}/source`);
}

/**
 * Persist a user-authored source override ("snapshot") for an addon. The
 * override is stored on disk under the user addons dir and survives restart.
 */
export async function saveAddonSource(id: string, source: string): Promise<AddonDetail> {
  return requestJson<AddonDetail>(`/api/addons/${encodeURIComponent(id)}/source`, {
    method: 'PUT',
    body: JSON.stringify({ source }),
  });
}

/** Drop a source override and restore the pristine builtin source. */
export async function resetAddonSource(id: string): Promise<AddonDetail> {
  return requestJson<AddonDetail>(`/api/addons/${encodeURIComponent(id)}/source`, {
    method: 'DELETE',
  });
}

/**
 * Install an addon packaged as a zip (`<name>.json` + `<name>.js`). The zip
 * bytes are sent as the raw request body; the server validates the manifest
 * `type` against the supported groups and rejects anything else.
 */
export async function uploadAddonZip(file: Blob): Promise<AddonInfo> {
  const response = await fetch('/api/addons/upload', {
    method: 'POST',
    headers: { 'Content-Type': 'application/zip' },
    body: file,
  });
  if (!response.ok) {
    throw new Error(await readErrorMessage(response));
  }
  return (await response.json()) as AddonInfo;
}

/** Permanently delete an uploaded (non-builtin) addon. */
export async function deleteAddon(id: string): Promise<void> {
  const response = await fetch(`/api/addons/${encodeURIComponent(id)}`, {
    method: 'DELETE',
  });
  if (!response.ok && response.status !== 204) {
    throw new Error(await readErrorMessage(response));
  }
}

/**
 * Download an addon as a zip holding its manifest (`<id>.json`) and source
 * (`<id>.js`) — the same layout the upload endpoint accepts, so the archive
 * can be edited offline and re-uploaded as-is.
 */
export async function exportAddonZip(id: string): Promise<Blob> {
  const response = await fetch(`/api/addons/${encodeURIComponent(id)}/export`);
  if (!response.ok) {
    throw new Error(await readErrorMessage(response));
  }
  return response.blob();
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
