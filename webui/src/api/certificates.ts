import type { CertificateStatus } from '@/types';

export async function getCertificateStatus(): Promise<CertificateStatus> {
  return request<CertificateStatus>('/api/certificates');
}

export async function getRootCaPem(): Promise<string> {
  const response = await fetch('/api/certificates/root-ca', { cache: 'no-store' });
  if (!response.ok) {
    throw new Error(`Request failed with status ${response.status}`);
  }
  return response.text();
}

export async function updateCertificateSans(extraSans: string[]): Promise<CertificateStatus> {
  return request<CertificateStatus>('/api/certificates/sans', {
    method: 'PUT',
    body: JSON.stringify({ extraSans }),
  });
}

export async function reissueCertificate(): Promise<CertificateStatus> {
  return request<CertificateStatus>('/api/certificates/reissue', {
    method: 'POST',
    body: '{}',
  });
}

async function request<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...init?.headers,
    },
  });
  if (!response.ok) {
    const text = await response.text();
    try {
      const data = JSON.parse(text) as { error?: string };
      throw new Error(data.error ?? text);
    } catch (error) {
      if (error instanceof SyntaxError) {
        throw new Error(text || `Request failed with status ${response.status}`);
      }
      throw error;
    }
  }
  return (await response.json()) as T;
}
