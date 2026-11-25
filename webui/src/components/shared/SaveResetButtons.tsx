import { Loader2, Save, RotateCcw } from 'lucide-react';
import { useTranslation } from '@/i18n';
import { cn } from '@/lib/cn';

interface SaveResetButtonsProps {
  onSave: () => void;
  onReset: () => void;
  saving?: boolean;
  isDirty?: boolean;
  /**
   * Extra gate for the save button on top of `isDirty` — e.g. the form
   * holds invalid values (like an empty MCP access token). The unsaved
   * hint still reflects `isDirty`.
   */
  saveDisabled?: boolean;
  saveLabel?: string;
  resetLabel?: string;
  className?: string;
}

export function SaveResetButtons({
  onSave,
  onReset,
  saving,
  isDirty,
  saveDisabled,
  saveLabel,
  resetLabel,
  className,
}: SaveResetButtonsProps) {
  const { t } = useTranslation();

  return (
    <div className={cn('flex items-center gap-3 pt-4', className)}>
      <button
        type="button"
        onClick={onSave}
        disabled={saving || !isDirty || saveDisabled}
        className={cn(
          'inline-flex items-center justify-center gap-2 px-4 py-1.5 rounded-standard text-xs font-semibold',
          'bg-amber-gold text-text-inverse border border-amber-gold',
          'hover:opacity-90 transition-opacity duration-vt-base',
          'disabled:opacity-50 disabled:cursor-not-allowed',
        )}
      >
        {saving ? (
          <Loader2 className="w-3.5 h-3.5 animate-spin" />
        ) : (
          <Save className="w-3.5 h-3.5" />
        )}
        {saveLabel ?? t('common.save')}
      </button>
      <button
        type="button"
        onClick={onReset}
        disabled={saving || !isDirty}
        className={cn(
          'inline-flex items-center justify-center gap-2 px-4 py-1.5 rounded-standard text-xs font-semibold',
          'bg-golden-cream text-warm-black border border-golden-sand',
          'hover:bg-pale-sand transition-colors duration-vt-base',
          'disabled:opacity-50 disabled:cursor-not-allowed',
        )}
      >
        <RotateCcw className="w-3.5 h-3.5" />
        {resetLabel ?? t('common.reset')}
      </button>
      {isDirty && (
        <span className="text-[11px] font-normal text-accent-text">
          {t('common.unsaved')}
        </span>
      )}
    </div>
  );
}
