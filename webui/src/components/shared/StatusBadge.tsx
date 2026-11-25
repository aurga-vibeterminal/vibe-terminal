import { cn } from '@/lib/cn';
import type { ConnectionStatus } from '@/context/connection-context';

interface StatusBadgeProps {
  status: ConnectionStatus | 'active' | 'inactive' | 'syncing' | string;
  label?: string;
  className?: string;
}

const statusStyles: Record<string, { bg: string; dot: string; text: string }> = {
  connected: { bg: 'var(--color-accent-subtle)', dot: 'var(--color-success)', text: 'var(--color-success)' },
  active: { bg: 'var(--color-accent-subtle)', dot: 'var(--color-success)', text: 'var(--color-success)' },
  disconnected: { bg: 'rgba(191,53,41,0.08)', dot: 'var(--color-error)', text: 'var(--color-error)' },
  inactive: { bg: 'var(--color-pale-sand)', dot: 'var(--color-warm-khaki)', text: 'var(--color-deep-olive)' },
  reconnecting: { bg: 'rgba(196,122,10,0.08)', dot: 'var(--color-warning)', text: 'var(--color-warning)' },
  syncing: { bg: 'rgba(45,107,158,0.08)', dot: 'var(--color-info)', text: 'var(--color-info)' },
};

export function StatusBadge({ status, label, className }: StatusBadgeProps) {
  const styles = statusStyles[status] ?? { bg: 'var(--color-pale-sand)', dot: 'var(--color-warm-khaki)', text: 'var(--color-deep-olive)' };

  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 px-2 py-0.5 rounded-pill text-[11px] font-medium',
        className,
      )}
      style={{
        backgroundColor: styles.bg,
        color: styles.text,
      }}
    >
      <span
        className="w-1.5 h-1.5 rounded-full"
        style={{ backgroundColor: styles.dot }}
      />
      {label ?? status}
    </span>
  );
}
