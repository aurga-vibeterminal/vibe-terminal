import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * Merge class values, resolving conflicting Tailwind utilities so the last
 * one wins (`cn('w-full', 'w-40')` → `w-40`). Without twMerge both classes
 * would apply and stylesheet order — not call order — would decide.
 */
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
