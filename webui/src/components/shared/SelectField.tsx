import { useState, useRef, useEffect, useCallback } from 'react';
import { ChevronDown } from 'lucide-react';
import { cn } from '@/lib/cn';

const MAX_VISIBLE_ITEMS = 6;
const ITEM_HEIGHT = 32;

export interface SelectFieldOption {
  value: string;
  label: string;
  disabled?: boolean;
}

interface SelectFieldProps {
  value: string;
  onChange: (e: { target: { value: string } }) => void;
  options: SelectFieldOption[];
  disabled?: boolean;
  className?: string;
  /** Dark variant: light text/controls for use on dark backgrounds. */
  dark?: boolean;
  /** Ghost variant: borderless transparent trigger that sizes to its content,
   *  for compact toolbar controls (e.g. the header language picker). */
  ghost?: boolean;
  /** Accessible label for the trigger button. */
  ariaLabel?: string;
}

export function SelectField({
  value,
  onChange,
  options,
  disabled,
  className,
  dark = false,
  ghost = false,
  ariaLabel,
}: SelectFieldProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const listRef = useRef<HTMLUListElement>(null);
  const selected = options.find((o) => o.value === value);

  const close = useCallback(() => setOpen(false), []);

  useEffect(() => {
    if (!open) return;
    function handleClickOutside(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        close();
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [open, close]);

  useEffect(() => {
    if (!open) return;
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === 'Escape') close();
    }
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [open, close]);

  useEffect(() => {
    if (!open || !listRef.current) return;
    const idx = options.findIndex((o) => o.value === value);
    if (idx >= 0) {
      listRef.current.scrollTop = idx * ITEM_HEIGHT;
    }
    // Intentionally runs only when the dropdown opens: `options` is rebuilt
    // on every parent render (a new array identity), so depending on it
    // would reset the scroll position on each data refresh while the user
    // is browsing — e.g. a downloading model's progress label updating on
    // every poll would yank the list back to the selected item.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const needsScroll = options.length > MAX_VISIBLE_ITEMS;
  const listMaxHeight = MAX_VISIBLE_ITEMS * ITEM_HEIGHT;

  // Determine dropdown placement: show above if not enough room below
  const [dropUp, setDropUp] = useState(false);
  useEffect(() => {
    if (!open || !ref.current) return;
    const rect = ref.current.getBoundingClientRect();
    const spaceBelow = window.innerHeight - rect.bottom;
    setDropUp(spaceBelow < 200 && rect.top > spaceBelow);
  }, [open]);

  return (
    <div ref={ref} className={cn('relative', ghost ? 'inline-block' : 'w-full', className)}>
      <button
        type="button"
        disabled={disabled}
        onClick={() => !disabled && setOpen(!open)}
        aria-label={ariaLabel}
        aria-haspopup="listbox"
        aria-expanded={open}
        className={cn(
          'flex items-center justify-between gap-2',
          'transition-colors duration-vt-base',
          'focus:outline-none focus-visible:ring-1 focus-visible:ring-amber-gold disabled:opacity-50 disabled:cursor-not-allowed',
          ghost
            ? cn(
                'w-full px-1 py-0.5 rounded-standard',
                dark ? 'text-golden-cream hover:bg-white/10' : 'text-deep-olive hover:text-warm-black',
              )
            : cn(
                'w-full px-2 py-1.5 text-sm font-normal rounded-standard border',
                dark
                  ? 'bg-white/5 text-golden-cream border-white/15'
                  : 'bg-bg-inset text-warm-black border-golden-sand',
                open && 'border-amber-gold',
              ),
        )}
      >
        <span className="truncate">{selected?.label ?? value}</span>
        <ChevronDown
          className={cn(
            'w-3.5 h-3.5 shrink-0 transition-transform duration-vt-base',
            dark ? 'text-amber-gold/70' : 'text-warm-khaki',
            open && 'rotate-180',
          )}
        />
      </button>
      {open && (
        <ul
          ref={listRef}
          role="listbox"
          className={cn(
            'absolute z-50 py-0.5 vt-scrollbar',
            ghost ? 'right-0 w-44 min-w-full' : 'w-full',
            dropUp ? 'bottom-full mb-1' : 'mt-1',
            'border rounded-comfortable shadow-md',
            dark ? 'bg-warm-black border-amber-gold/40' : 'bg-bg-inset border-golden-sand',
          )}
          style={{
            maxHeight: `${listMaxHeight}px`,
            overflowY: needsScroll ? 'auto' : undefined,
            WebkitOverflowScrolling: 'touch',
          }}
        >
          {options.map((opt) => (
            <li key={opt.value}>
              <button
                type="button"
                role="option"
                aria-selected={opt.value === value}
                disabled={opt.disabled}
                onClick={() => {
                  if (opt.disabled) return;
                  onChange({ target: { value: opt.value } });
                  close();
                }}
                className={cn(
                  'w-full text-left px-3 py-1.5 text-sm transition-colors duration-vt-base',
                  'disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:bg-transparent',
                  opt.value === value
                    ? dark
                      ? 'bg-amber-gold/20 text-amber-gold font-medium'
                      : 'bg-accent-subtle text-accent-text font-medium'
                    : dark
                      ? 'text-golden-cream hover:bg-white/10'
                      : 'text-warm-black hover:bg-bg-active',
                )}
              >
                {opt.label}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
