import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  AlertCircle,
  AlertTriangle,
  Copy,
  Download,
  Loader2,
  Plus,
  RefreshCw,
  ShieldCheck,
  Trash2,
} from 'lucide-react';
import { getCertificateStatus, getRootCaPem, reissueCertificate, updateCertificateSans } from '@/api/certificates';
import { SectionHeader } from '@/components/shared/SectionHeader';
import { StatusBadge } from '@/components/shared/StatusBadge';
import { useToast } from '@/components/shared/Toast';
import { useTranslation } from '@/i18n';
import { copyToClipboard } from '@/utils/clipboard';
import type { CertificateInfo, CertificateStatus } from '@/types';

export function CertificatesSection() {
  const { t } = useTranslation();
  const { addToast } = useToast();
  const [status, setStatus] = useState<CertificateStatus | null>(null);
  const [extraSans, setExtraSans] = useState<string[]>([]);
  const [newSan, setNewSan] = useState('');
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState<'save' | 'reissue' | 'copy' | null>(null);
  const [error, setError] = useState<string | null>(null);

  const applyStatus = useCallback((next: CertificateStatus) => {
    setStatus(next);
    setExtraSans(next.extraSans);
  }, []);

  const load = useCallback(async () => {
    setError(null);
    try {
      applyStatus(await getCertificateStatus());
    } catch (loadError) {
      setError(loadError instanceof Error ? loadError.message : t('certificates.loadError'));
    } finally {
      setLoading(false);
    }
  }, [applyStatus, t]);

  useEffect(() => {
    void load();
  }, [load]);

  const dirty = useMemo(
    () => JSON.stringify(extraSans) !== JSON.stringify(status?.extraSans ?? []),
    [extraSans, status?.extraSans],
  );
  const previewSans = useMemo(
    () =>
      Array.from(
        new Set([
          ...(status?.automaticSans ?? []),
          ...extraSans.map((value) => value.trim()).filter(Boolean),
        ]),
      ).sort(),
    [extraSans, status?.automaticSans],
  );

  const handleAdd = () => {
    const value = newSan.trim();
    if (!value) return;
    if (!extraSans.some((item) => item.toLowerCase() === value.toLowerCase())) {
      setExtraSans((items) => [...items, value]);
    }
    setNewSan('');
  };

  const handleSave = async () => {
    setBusy('save');
    setError(null);
    try {
      applyStatus(await updateCertificateSans(extraSans));
      addToast('success', t('certificates.sansSaved'));
    } catch (saveError) {
      const message = saveError instanceof Error ? saveError.message : t('certificates.saveError');
      setError(message);
      addToast('error', message);
    } finally {
      setBusy(null);
    }
  };

  const handleReissue = async () => {
    setBusy('reissue');
    setError(null);
    try {
      applyStatus(await reissueCertificate());
      addToast('success', t('certificates.reissued'));
    } catch (reissueError) {
      const message = reissueError instanceof Error ? reissueError.message : t('certificates.reissueError');
      setError(message);
      addToast('error', message);
    } finally {
      setBusy(null);
    }
  };

  const handleCopyCa = async () => {
    setBusy('copy');
    try {
      const pem = await getRootCaPem();
      const copied = await copyToClipboard(pem);
      addToast(copied ? 'success' : 'error', t(copied ? 'certificates.caCopied' : 'certificates.caCopyFailed'));
    } catch {
      addToast('error', t('certificates.caCopyFailed'));
    } finally {
      setBusy(null);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-6 h-6 animate-spin text-warm-black" />
      </div>
    );
  }

  if (!status) {
    return (
      <div className="flex items-center gap-2 text-warm-black py-8">
        <AlertCircle className="w-5 h-5" />
        <span className="font-light">{error ?? t('certificates.loadError')}</span>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <SectionHeader title={t('certificates.title')} description={t('certificates.description')} />

      <div className="flex items-start gap-2 rounded-standard border border-vt-warning bg-vt-warning/25 px-3 py-2">
        <AlertTriangle className="w-4 h-4 shrink-0 mt-0.5 text-warm-black" />
        <p className="text-xs text-warm-black leading-relaxed">{t('certificates.trustWarning')}</p>
      </div>

      {error && (
        <div className="flex items-start gap-2 text-xs text-vt-error">
          <AlertCircle className="w-4 h-4 shrink-0" />
          <span>{error}</span>
        </div>
      )}

      <section className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <div className="flex items-start justify-between gap-4">
          <div className="flex items-center gap-2">
            <ShieldCheck className="w-4 h-4 text-deep-olive" />
            <h3 className="text-sm font-semibold text-warm-black">{t('certificates.rootCa')}</h3>
          </div>
          {status.rootCa && (
            <div className="flex flex-wrap justify-end gap-2">
              <a
                href="/api/certificates/root-ca"
                download="vibe-terminal-root-ca.pem"
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-standard text-xs font-semibold bg-amber-gold text-text-inverse border border-amber-gold hover:opacity-90"
              >
                <Download className="w-3.5 h-3.5" />
                {t('certificates.downloadCa')}
              </a>
              <button
                type="button"
                onClick={() => void handleCopyCa()}
                disabled={busy !== null}
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-standard text-xs font-medium text-deep-olive border border-golden-sand hover:bg-pale-sand disabled:opacity-50"
              >
                {busy === 'copy' ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Copy className="w-3.5 h-3.5" />}
                {t('certificates.copyCa')}
              </button>
            </div>
          )}
        </div>

        {status.rootCa ? (
          <CertificateDetails certificate={status.rootCa} />
        ) : (
          <p className="text-xs text-warm-khaki">{t('certificates.noRootCa')}</p>
        )}
        <p className="text-xs text-warm-khaki leading-relaxed">{t('certificates.caPublicOnly')}</p>
      </section>

      <section className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-5 border border-golden-sand">
        <div>
          <h3 className="text-sm font-semibold text-warm-black">{t('certificates.sanManagement')}</h3>
          <p className="text-xs text-warm-khaki mt-1">{t('certificates.sanDescription')}</p>
        </div>

        <div>
          <h4 className="text-xs font-medium text-warm-khaki mb-2">{t('certificates.automaticSans')}</h4>
          <SanList values={status.automaticSans} />
        </div>

        <div className="space-y-3">
          <h4 className="text-xs font-medium text-warm-khaki">{t('certificates.extraSans')}</h4>
          {extraSans.map((value, index) => (
            <div key={index} className="flex items-center gap-2">
              <input
                type="text"
                value={value}
                onChange={(event) =>
                  setExtraSans((items) => items.map((item, itemIndex) => (itemIndex === index ? event.target.value : item)))
                }
                className="flex-1 min-w-0 px-2 py-1.5 bg-bg-inset border border-golden-sand rounded-standard text-warm-black text-xs font-mono focus:outline-none focus:border-amber-gold"
                aria-label={`${t('certificates.extraSans')} ${index + 1}`}
              />
              <button
                type="button"
                onClick={() => setExtraSans((items) => items.filter((_, itemIndex) => itemIndex !== index))}
                className="p-1.5 rounded-standard text-deep-olive hover:text-vt-error hover:bg-pale-sand"
                title={t('common.remove')}
                aria-label={t('common.remove')}
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          ))}
          <div className="flex items-center gap-2">
            <input
              type="text"
              value={newSan}
              onChange={(event) => setNewSan(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === 'Enter') {
                  event.preventDefault();
                  handleAdd();
                }
              }}
              placeholder={t('certificates.sanPlaceholder')}
              className="flex-1 min-w-0 px-2 py-1.5 bg-bg-inset border border-golden-sand rounded-standard text-warm-black text-xs font-mono placeholder:text-warm-khaki focus:outline-none focus:border-amber-gold"
            />
            <button
              type="button"
              onClick={handleAdd}
              disabled={!newSan.trim()}
              className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-standard text-xs font-medium border border-golden-sand text-deep-olive hover:bg-pale-sand disabled:opacity-50"
            >
              <Plus className="w-3.5 h-3.5" />
              {t('certificates.addSan')}
            </button>
          </div>
        </div>

        <div>
          <h4 className="text-xs font-medium text-warm-khaki mb-2">{t('certificates.sanPreview')}</h4>
          <SanList values={previewSans} />
          <p className="text-xs text-warm-khaki mt-2">{t('certificates.sanMatchHint')}</p>
        </div>

        <div className="border-t border-golden-sand/60 pt-4 flex flex-wrap items-center gap-3">
          <button
            type="button"
            onClick={() => void handleSave()}
            disabled={!dirty || busy !== null}
            className="inline-flex items-center gap-2 px-4 py-1.5 bg-amber-gold text-text-inverse rounded-standard font-medium text-xs hover:opacity-90 disabled:opacity-50 border border-amber-gold"
          >
            {busy === 'save' && <Loader2 className="w-3.5 h-3.5 animate-spin" />}
            {t('certificates.saveAndReissue')}
          </button>
          <button
            type="button"
            onClick={() => void handleReissue()}
            disabled={dirty || busy !== null}
            className="inline-flex items-center gap-2 px-4 py-1.5 text-deep-olive rounded-standard font-medium text-xs hover:bg-pale-sand disabled:opacity-50 border border-golden-sand"
          >
            {busy === 'reissue' ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <RefreshCw className="w-3.5 h-3.5" />}
            {status.rootCa ? t('certificates.reissue') : t('certificates.generateCa')}
          </button>
        </div>
      </section>

      <section className="bg-golden-cream rounded-comfortable p-4 md:p-6 space-y-4 border border-golden-sand">
        <div className="flex items-center justify-between gap-4">
          <h3 className="text-sm font-semibold text-warm-black">{t('certificates.leafCertificate')}</h3>
          <StatusBadge
            status={status.lastError ? 'disconnected' : status.leafCertificate ? 'active' : 'inactive'}
            label={
              status.lastError
                ? t('certificates.statusError')
                : status.leafCertificate
                  ? t('certificates.statusValid')
                  : t('certificates.statusMissing')
            }
          />
        </div>
        {!status.managedIdentityActive && (
          <p className="text-xs text-warm-khaki">{t('certificates.customIdentityActive')}</p>
        )}
        {status.leafCertificate ? (
          <CertificateDetails certificate={status.leafCertificate} />
        ) : (
          <p className="text-xs text-warm-khaki">{t('certificates.noLeafCertificate')}</p>
        )}
        {status.lastError && <p className="text-xs text-vt-error break-words">{status.lastError}</p>}
        <p className="text-xs text-warm-khaki leading-relaxed">{t('certificates.renewalHint')}</p>
      </section>
    </div>
  );
}

function CertificateDetails({ certificate }: { certificate: CertificateInfo }) {
  const { t } = useTranslation();
  return (
    <dl className="grid grid-cols-1 sm:grid-cols-[120px,1fr] gap-x-4 gap-y-3 text-xs">
      <dt className="text-warm-khaki">{t('certificates.subject')}</dt>
      <dd className="text-warm-black break-all">{certificate.subject}</dd>
      <dt className="text-warm-khaki">{t('certificates.fingerprint')}</dt>
      <dd className="text-warm-black font-mono break-all select-all">{certificate.fingerprintSha256}</dd>
      <dt className="text-warm-khaki">{t('certificates.validFrom')}</dt>
      <dd className="text-warm-black">{formatDate(certificate.notBefore)}</dd>
      <dt className="text-warm-khaki">{t('certificates.validUntil')}</dt>
      <dd className="text-warm-black">{formatDate(certificate.notAfter)}</dd>
      {!certificate.isCa && (
        <>
          <dt className="text-warm-khaki">{t('certificates.sans')}</dt>
          <dd><SanList values={certificate.sans} /></dd>
        </>
      )}
    </dl>
  );
}

function SanList({ values }: { values: string[] }) {
  const { t } = useTranslation();
  if (values.length === 0) {
    return <span className="text-xs text-warm-khaki">{t('certificates.noSans')}</span>;
  }
  return (
    <div className="flex flex-wrap gap-1.5">
      {values.map((value) => (
        <span key={value} className="px-2 py-1 rounded-standard bg-bg-inset border border-golden-sand text-warm-black text-[11px] font-mono break-all">
          {value}
        </span>
      ))}
    </div>
  );
}

function formatDate(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleString();
}
