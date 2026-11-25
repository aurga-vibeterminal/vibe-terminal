import { cn } from '@/lib/cn';

interface NumberInputProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'onChange' | 'type'> {
  value: number;
  onChange: (value: number) => void;
}

export function NumberInput({ value, onChange, className, ...props }: NumberInputProps) {
  return (
    <input
      type="number"
      value={value}
      onChange={(e) => onChange(Number(e.target.value))}
      className={cn(
        'w-full',
        className,
      )}
      {...props}
    />
  );
}
