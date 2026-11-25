export interface CertificateInfo {
  subject: string;
  fingerprintSha256: string;
  notBefore: string;
  notAfter: string;
  sans: string[];
  isCa: boolean;
}

export interface CertificateStatus {
  rootCa: CertificateInfo | null;
  leafCertificate: CertificateInfo | null;
  automaticSans: string[];
  extraSans: string[];
  sanPreview: string[];
  managedIdentityActive: boolean;
  lastError: string | null;
}
