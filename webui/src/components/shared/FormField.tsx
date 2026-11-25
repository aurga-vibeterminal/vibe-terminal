import type { ReactNode } from 'react';
import { cn } from '@/lib/cn';

interface FormFieldProps {
  label: string;
  htmlFor?: string;
  description?: string;
  error?: string;
  children: ReactNode;
  className?: string;
}

export function FormField({ label, htmlFor, description, error, children, className }: FormFieldProps) {
  return (
    <div className={cn('space-y-1.5', className)}>
      <label
        htmlFor={htmlFor}
        className="block text-xs font-medium text-warm-black"
      >
        {label}
      </label>
      {description && (
        <p
          className="text-xs font-normal text-warm-khaki"
        >
          {description}
        </p>
      )}
      {children}
      {error && (
        <p className="text-xs font-normal text-vt-error">
          {error}
        </p>
      )}
    </div>
  );
}
