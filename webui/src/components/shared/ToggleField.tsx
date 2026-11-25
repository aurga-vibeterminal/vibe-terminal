import { cn } from '@/lib/cn';

interface ToggleFieldProps {
  enabled: boolean;
  onChange: (value: boolean) => void;
  label?: string;
  disabled?: boolean;
  className?: string;
}

export function ToggleField({ enabled, onChange, label, disabled, className }: ToggleFieldProps) {
  return (
    <label
      className={cn(
        'inline-flex items-center gap-3 cursor-pointer',
        disabled && 'opacity-50 cursor-not-allowed',
        className,
      )}
    >
      <button
        type="button"
        role="switch"
        aria-checked={enabled}
        disabled={disabled}
        onClick={() => onChange(!enabled)}
        className={cn(
          'relative inline-flex h-5 w-9 shrink-0 rounded-pill transition-colors duration-vt-base',
          'focus:outline-none focus:ring-2 focus:ring-amber-gold focus:ring-offset-1 focus:ring-offset-golden-cream',
          enabled ? 'bg-amber-gold' : 'bg-golden-sand',
        )}
      >
        <span
          className={cn(
            'pointer-events-none inline-block h-4 w-4 rounded-full bg-golden-cream shadow-sm ring-0 transition-transform duration-vt-base translate-y-0.5',
            enabled ? 'translate-x-[18px]' : 'translate-x-0.5',
          )}
        />
      </button>
      {label && (
        <span
          className="text-xs font-normal text-warm-black"
        >
          {label}
        </span>
      )}
    </label>
  );
}
