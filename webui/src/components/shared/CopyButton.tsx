import { useCallback, useState } from 'react';
import { Check, Copy } from 'lucide-react';
import { useTranslation } from '@/i18n';

interface CopyButtonProps {
  /** Text to copy to the clipboard. */
  text: string;
  /** Accessibility title / tooltip. Defaults to a generic "Copy" label. */
  title?: string;
  className?: string;
}

/**
 * Icon button that copies `text` to the system clipboard, swapping to a green
 * check for 2 seconds on success. Mirrors the inline component that previously
 * lived in VoiceSection; lifted here so McpSection (and others) can reuse it.
 */
export function CopyButton({ text, title, className }: CopyButtonProps) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);
  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(text).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, [text]);
  return (
    <button
      type="button"
      onClick={handleCopy}
      className={
        className ??
        'p-1 rounded-standard text-deep-olive hover:text-warm-black hover:bg-golden-cream transition-colors duration-vt-base'
      }
      title={title ?? t('common.copy')}
    >
      {copied ? (
        <Check className="w-3.5 h-3.5 text-green-600" />
      ) : (
        <Copy className="w-3.5 h-3.5" />
      )}
    </button>
  );
}
