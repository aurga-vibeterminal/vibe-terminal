import type {
  ScreenConfiguredPaths,
  ScreenConfiguredStream,
  ScreenConsentResetResponse,
  ScreenEncoderResponse,
  ScreenInputStatusResponse,
  ScreenPathsResponse,
  ScreenServiceResponse,
} from '@/types';

export async function fetchScreenPaths(): Promise<ScreenPathsResponse> {
  return requestJson<ScreenPathsResponse>('/api/screen/paths');
}

export async function saveScreenPaths(
  paths: ScreenConfiguredPaths,
): Promise<ScreenPathsResponse> {
  return requestJson<ScreenPathsResponse>('/api/screen/paths', {
    method: 'PUT',
    body: JSON.stringify(paths),
  });
}

export async function fetchScreenEncoder(): Promise<ScreenEncoderResponse> {
  return requestJson<ScreenEncoderResponse>('/api/screen/encoder');
}

export async function saveScreenEncoder(
  stream: ScreenConfiguredStream,
): Promise<ScreenEncoderResponse> {
  return requestJson<ScreenEncoderResponse>('/api/screen/encoder', {
    method: 'PUT',
    body: JSON.stringify(stream),
  });
}

/** SCM status of the Windows service (installed/running). */
export async function fetchScreenService(): Promise<ScreenServiceResponse> {
  return requestJson<ScreenServiceResponse>('/api/screen/service');
}

/** Install the auto-start service (an elevated helper runs; UAC prompt). */
export async function installScreenService(): Promise<ScreenServiceResponse> {
  return requestJson<ScreenServiceResponse>('/api/screen/service/install', {
    method: 'POST',
  });
}

/** Stop and delete the service (an elevated helper runs; UAC prompt). */
export async function removeScreenService(): Promise<ScreenServiceResponse> {
  return requestJson<ScreenServiceResponse>('/api/screen/service/remove', {
    method: 'POST',
  });
}

/** GET /api/screen/input — the Linux `/dev/uinput` probe (setup card). */
export async function fetchScreenInputStatus(): Promise<ScreenInputStatusResponse> {
  return requestJson<ScreenInputStatusResponse>('/api/screen/input');
}

/** POST /api/screen/consent/reset — re-ask which display to share next time. */
export async function resetScreenConsent(): Promise<ScreenConsentResetResponse> {
  return requestJson<ScreenConsentResetResponse>('/api/screen/consent/reset', {
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
