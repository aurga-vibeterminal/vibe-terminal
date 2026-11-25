import { useState } from 'react';
import { Dices, Eye, EyeOff } from 'lucide-react';
import { CopyButton } from './CopyButton';

const inputClassName = [
  'w-full bg-bg-inset px-2 py-1.5 text-sm font-normal',
  'rounded-standard border border-golden-sand',
  'transition-colors duration-vt-base',
  'focus:outline-none focus:border-amber-gold',
  'disabled:opacity-50 disabled:cursor-not-allowed',
  'text-warm-black',
].join(' ');

interface SecretFieldProps extends React.InputHTMLAttributes<HTMLInputElement> {
  placeholder?: string;
  /**
   * When non-empty, a copy button appears beside the eye that copies this
   * text straight to the clipboard — without revealing the field — so a
   * stored secret never has to be shown to be handed out. Empty string
   * (nothing saved yet) hides the button.
   */
  copyText?: string;
  /** Tooltip for the copy button. */
  copyTitle?: string;
  /** When provided, a dice button writes a fresh random value into the field. */
  onGenerate?: () => void;
  /** Tooltip for the dice button. */
  generateTitle?: string;
}

export function SecretField({
  value,
  onChange,
  placeholder,
  disabled,
  className,
  copyText,
  copyTitle,
  onGenerate,
  generateTitle,
  ...rest
}: SecretFieldProps) {
  const [visible, setVisible] = useState(false);

  const showCopy = !!copyText;
  // Trailing icon row: [copy] eye [dice]. The dice keeps its rightmost spot;
  // input padding widens to whatever the row occupies (each icon button is
  // 22px + 2px gap, anchored right-2 → 40 / 64 / 96px for 1–3 buttons).
  const buttonCount = 1 + (showCopy ? 1 : 0) + (onGenerate ? 1 : 0);
  const padClass =
    buttonCount === 1 ? 'pr-10' : buttonCount === 2 ? 'pr-16' : 'pr-24';

  return (
    <div className="relative">
      <input
        type={visible ? 'text' : 'password'}
        value={value}
        onChange={onChange}
        className={className ?? `${inputClassName} ${padClass}`}
        placeholder={placeholder}
        disabled={disabled}
        {...rest}
      />
      <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-0.5">
        {showCopy && (
          <CopyButton
            text={copyText}
            title={copyTitle}
            className="p-1 rounded-tight text-deep-olive hover:text-warm-black hover:bg-pale-sand transition-colors duration-vt-base"
          />
        )}
        <button
          type="button"
          onClick={() => setVisible((v) => !v)}
          disabled={disabled}
          className="p-1 rounded-tight text-deep-olive hover:text-warm-black hover:bg-pale-sand transition-colors duration-vt-base disabled:opacity-50 disabled:pointer-events-none"
          aria-label={visible ? 'Hide value' : 'Show value'}
        >
          {visible ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
        </button>
        {onGenerate && (
          <button
            type="button"
            onClick={onGenerate}
            disabled={disabled}
            className="p-1 rounded-tight text-deep-olive hover:text-warm-black hover:bg-pale-sand transition-colors duration-vt-base disabled:opacity-50 disabled:pointer-events-none"
            aria-label={generateTitle}
            title={generateTitle}
          >
            <Dices className="w-3.5 h-3.5" />
          </button>
        )}
      </div>
    </div>
  );
}
