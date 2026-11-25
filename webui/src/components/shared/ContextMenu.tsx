import { useEffect } from 'react';
import { cn } from '@/lib/cn';

export interface ContextMenuItem {
  /** Stable key. */
  id: string;
  label: string;
  icon?: React.ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  /** Render a thin separator above this item. */
  separatorBefore?: boolean;
  danger?: boolean;
}

interface ContextMenuProps {
  open: boolean;
  /** Viewport coordinates (clientX/clientY). */
  x: number;
  y: number;
  items: ContextMenuItem[];
  onClose: () => void;
}

/**
 * A lightweight right-click context menu. Renders a fixed overlay that captures
 * the next outside click / scroll / escape / resize to close itself, and
 * positions the panel at `(x, y)`, clamping to the viewport.
 */
export function ContextMenu({ open, x, y, items, onClose }: ContextMenuProps) {
  useEffect(() => {
    if (!open) return;
    const close = () => onClose();
    const onOutsideMouseDown = (e: MouseEvent) => {
      // If the user clicks inside the menu we let the item handler run; any
      // other click closes.
      const target = e.target as HTMLElement | null;
      if (target && target.closest('[data-context-menu]')) return;
      onClose();
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') close();
    };
    // Defer attaching the mousedown/contextmenu listeners so the event that
    // opened us does not immediately close us again.
    const id = window.setTimeout(() => {
      window.addEventListener('mousedown', onOutsideMouseDown, true);
      window.addEventListener('contextmenu', close, true);
      window.addEventListener('scroll', close, true);
      window.addEventListener('resize', close);
      window.addEventListener('blur', close);
    }, 0);
    window.addEventListener('keydown', onKey);
    return () => {
      window.clearTimeout(id);
      window.removeEventListener('mousedown', onOutsideMouseDown, true);
      window.removeEventListener('contextmenu', close, true);
      window.removeEventListener('scroll', close, true);
      window.removeEventListener('resize', close);
      window.removeEventListener('blur', close);
      window.removeEventListener('keydown', onKey);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, onClose]);

  if (!open) return null;

  // Clamp to viewport so the menu never overflows. The panel width/height are
  // estimated; the menu will reposition on the next interaction if needed.
  const estWidth = 200;
  const estHeight = items.length * 32 + 8;
  const clampedX = Math.min(x, window.innerWidth - estWidth - 8);
  const clampedY = Math.min(y, window.innerHeight - estHeight - 8);

  return (
    <div className="fixed inset-0 z-50">
      <div
        data-context-menu
        role="menu"
        className="absolute min-w-[180px] py-1 rounded-standard bg-warm-ivory border border-golden-sand shadow-md"
        style={{ left: Math.max(8, clampedX), top: Math.max(8, clampedY) }}
        onMouseDown={(e) => e.stopPropagation()}
      >
        {items.map((item, index) => (
          <div key={item.id}>
            {item.separatorBefore && index > 0 && (
              <div className="my-1 mx-2 border-t border-golden-sand/60" />
            )}
            <button
              type="button"
              role="menuitem"
              disabled={item.disabled}
              onClick={() => {
                if (item.disabled) return;
                item.onClick?.();
                onClose();
              }}
              className={cn(
                'w-full flex items-center gap-2 px-3 py-1.5 text-xs text-left',
                'transition-colors duration-vt-base',
                item.disabled
                  ? 'text-warm-khaki cursor-not-allowed'
                  : item.danger
                    ? 'text-vt-error hover:bg-vt-error/10'
                    : 'text-warm-black hover:bg-pale-sand',
              )}
            >
              {item.icon && <span className="shrink-0 inline-flex">{item.icon}</span>}
              <span className="truncate">{item.label}</span>
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
