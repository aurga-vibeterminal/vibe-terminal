import { useState } from 'react';
import { ExternalLink, Link2, Loader2, Square } from 'lucide-react';
import { createPortForwardShare } from '@/api/port-forwarding';
import { StatusBadge } from '@/components/shared/StatusBadge';
import { useToast } from '@/components/shared/Toast';
import { useTranslation } from '@/i18n';
import { copyToClipboard } from '@/utils/clipboard';
import type { PortForward, PortForwardShare } from '@/types';

interface PortForwardListProps {
  forwards: PortForward[];
  onStop: (proxyPort: number) => void;
}

interface ShareLink {
  targetPort: number;
  url: string;
  oneTime: boolean;
}

function buildShareUrl(proxyPort: number, share: PortForwardShare): string {
  const url = new URL(window.location.href);
  url.protocol = 'https:';
  url.port = String(proxyPort);
  url.pathname = '/';
  url.search = '';
  url.hash = '';
  if (share.shareToken) {
    url.searchParams.set('share_token', share.shareToken);
  } else if (share.ticket) {
    url.searchParams.set('ticket', share.ticket);
  }
  return url.toString();
}

export function PortForwardList({ forwards, onStop }: PortForwardListProps) {
  const { t } = useTranslation();
  const { addToast } = useToast();
  const [shareLinks, setShareLinks] = useState<Record<number, ShareLink>>({});
  const [sharingPort, setSharingPort] = useState<number | null>(null);

  const handleShare = async (forward: PortForward) => {
    setSharingPort(forward.proxyPort);
    try {
      const share = await createPortForwardShare(forward.proxyPort);
      const url = buildShareUrl(forward.proxyPort, share);
      setShareLinks((current) => ({
        ...current,
        [forward.proxyPort]: {
          targetPort: forward.targetPort,
          url,
          oneTime: Boolean(share.ticket),
        },
      }));
      const copied = await copyToClipboard(url);
      addToast(
        copied ? 'success' : 'error',
        t(copied ? 'portForward.shareLinkCopied' : 'portForward.shareLinkCopyFailed'),
      );
    } catch (error) {
      const message = error instanceof Error && error.message ? error.message : null;
      addToast('error', message ?? t('portForward.shareLinkFailed'));
    } finally {
      setSharingPort(null);
    }
  };

  if (forwards.length === 0) {
    return (
      <div className="text-center py-6 text-warm-khaki text-xs">
        {t('portForward.noForwards')}
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {forwards.map((forward) => {
        const savedLink = shareLinks[forward.proxyPort];
        const shareLink = savedLink?.targetPort === forward.targetPort ? savedLink : null;
        return (
          <div
            key={forward.proxyPort}
            className="p-3 rounded-standard bg-golden-cream border border-golden-sand transition-all duration-vt-base space-y-3"
          >
            <div className="flex items-center justify-between gap-3">
              <div className="min-w-0">
                <div className="flex items-center gap-2">
                  <span className="font-medium text-warm-black text-xs">
                    :{forward.proxyPort}
                    <span className="mx-1 text-warm-khaki">&rarr;</span>
                    :{forward.targetPort}
                  </span>
                  <StatusBadge status="active" label={t('portForward.listening')} />
                </div>
                <div className="flex items-center gap-3 mt-1 text-xs text-deep-olive">
                  <span>
                    {t('portForward.activeConnections', { count: forward.activeConnections })}
                  </span>
                </div>
              </div>
              <div className="flex items-center gap-1 shrink-0">
                <button
                  type="button"
                  disabled={sharingPort !== null}
                  onClick={() => void handleShare(forward)}
                  className="inline-flex items-center gap-1.5 px-2 py-1.5 rounded-standard text-xs text-deep-olive hover:text-warm-black hover:bg-pale-sand transition-colors duration-vt-base disabled:opacity-50"
                  title={t('portForward.copyShareLink')}
                >
                  {sharingPort === forward.proxyPort ? (
                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  ) : (
                    <Link2 className="w-3.5 h-3.5" />
                  )}
                  {t('portForward.copyShareLink')}
                </button>
                <button
                  type="button"
                  onClick={() => onStop(forward.proxyPort)}
                  className="p-1.5 rounded-standard text-warm-khaki hover:text-vt-error hover:bg-pale-sand transition-colors duration-vt-base"
                  title={t('portForward.stop')}
                >
                  <Square className="w-3 h-3 fill-current" />
                </button>
              </div>
            </div>
            {shareLink && (
              <div className="rounded-standard border border-golden-sand bg-pale-sand/40 px-3 py-2 space-y-1.5">
                <div className="flex items-center justify-between gap-2">
                  <span className="text-xs font-medium text-deep-olive">
                    {t('portForward.shareLink')}
                  </span>
                  <a
                    href={shareLink.url}
                    target="_blank"
                    rel="noreferrer"
                    className="inline-flex items-center gap-1 text-xs text-deep-olive hover:text-warm-black"
                    title={t('portForward.openShareLink')}
                  >
                    {t('portForward.openShareLink')}
                    <ExternalLink className="w-3 h-3" />
                  </a>
                </div>
                <code className="block text-xs text-warm-black break-all select-all">
                  {shareLink.url}
                </code>
                {shareLink.oneTime && (
                  <p className="text-xs font-light text-warm-khaki">
                    {t('portForward.settingsTicketHint')}
                  </p>
                )}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
