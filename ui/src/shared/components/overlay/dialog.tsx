import { useEffect, useRef, type CSSProperties, type ReactNode } from "react";
import { createPortal } from "react-dom";

/**
 * A Material 3 dialog over a scrim; Escape and a click on the scrim dismiss it. `buttons` sit at the bottom right.
 */
export function Dialog({ title, children, buttons, onDismiss, width, style, className }: {
  title?: ReactNode;
  children: ReactNode;
  buttons?: ReactNode;
  onDismiss: () => void;
  width?: number | string;
  style?: CSSProperties;
  className?: string;
}) {
  const box = useRef<HTMLDivElement>(null);
  // The latest callback, so an inline `onDismiss` neither re-registers the key nor moves the focus on each render.
  const dismiss = useRef(onDismiss);
  dismiss.current = onDismiss;
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        dismiss.current();
      }
    };
    window.addEventListener("keydown", onKey);
    box.current?.querySelector<HTMLElement>("[autofocus], input, textarea, button")?.focus();
    return () => window.removeEventListener("keydown", onKey);
  }, []);
  return createPortal(
    <div className="scrim" onMouseDown={(event) => event.target === event.currentTarget && onDismiss()}>
      <div ref={box} role="dialog" aria-modal="true" className={className ? `dialog ${className}` : "dialog"} style={{ width, ...style }}>
        {title && <h2 className="dialog-title">{title}</h2>}
        {children}
        {buttons && <div className="dialog-buttons">{buttons}</div>}
      </div>
    </div>,
    document.body,
  );
}
