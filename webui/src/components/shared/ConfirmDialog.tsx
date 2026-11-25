import { AlertTriangle } from 'lucide-react';
import { cn } from '@/lib/cn';
import { useTranslation } from '@/i18n';

interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: 'danger' | 'default';
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  open,
  title,
  message,
  confirmLabel,
  cancelLabel,
  variant = 'default',
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const { t } = useTranslation();

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div
        className="fixed inset-0 bg-warm-black/50"
        onClick={onCancel}
        style={{ backdropFilter: 'blur(4px)' }}
      />
      <div
        className="relative max-w-md w-full mx-4 p-5 rounded-comfortable bg-warm-ivory border border-golden-sand shadow-lg"
      >
        <div className="flex items-start gap-4">
          {variant === 'danger' && (
            <div
              className="flex-shrink-0 w-9 h-9 rounded-full flex items-center justify-center bg-pale-sand"
            >
              <AlertTriangle className="w-4 h-4 text-vt-error" />
            </div>
          )}
          <div className="flex-1">
            <h3 className="text-sm font-semibold text-warm-black">
              {title}
            </h3>
            <p className="text-xs font-light mt-1.5 text-deep-olive">
              {message}
            </p>
          </div>
        </div>
        <div className="flex justify-end gap-3 mt-5">
          <button
            type="button"
            onClick={onCancel}
            className={cn(
              'inline-flex items-center justify-center px-4 py-1.5 rounded-standard text-xs font-medium',
              'bg-golden-cream text-warm-black border border-golden-sand',
              'hover:bg-pale-sand transition-colors duration-vt-base',
            )}
          >
            {cancelLabel ?? t('common.cancel')}
          </button>
          <button
            type="button"
            onClick={onConfirm}
            className={cn(
              'inline-flex items-center justify-center px-4 py-1.5 rounded-standard text-xs font-medium',
              'text-text-inverse hover:opacity-90 transition-opacity duration-vt-base',
              variant === 'danger'
                ? 'bg-vt-error border border-vt-error'
                : 'bg-amber-gold border border-amber-gold',
            )}
          >
            {confirmLabel ?? t('common.confirm')}
          </button>
        </div>
      </div>
    </div>
  );
}
