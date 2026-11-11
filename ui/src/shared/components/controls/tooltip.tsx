import {type ReactNode, useEffect, useRef, useState} from "react";
import {createPortal} from "react-dom";

const DELAY_MS = 450;
const GAP = 6;

/**
 * Shows `reason` as a tooltip above `children`, or to its right when `beside`; without a reason, `children`
 * alone. Disabled buttons get no pointer events in some engines, so the hover is taken on a wrapping span.
 */
export function ReasonTooltip({ reason, beside = false, children }: { reason: string | null | undefined; beside?: boolean; children: ReactNode }) {
  const anchor = useRef<HTMLSpanElement>(null);
  const [position, setPosition] = useState<{ left: number; top: number } | null>(null);
  const timer = useRef<number | undefined>(undefined);
  useEffect(() => () => window.clearTimeout(timer.current), []);
  if (!reason) return <>{children}</>;
  const show = () => {
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => {
      const rect = anchor.current?.getBoundingClientRect();
      if (!rect) return;
      setPosition(beside ? { left: rect.right + GAP, top: rect.top + rect.height / 2 } : { left: rect.left + rect.width / 2, top: rect.top - GAP });
    }, DELAY_MS);
  };
  const hide = () => {
    window.clearTimeout(timer.current);
    setPosition(null);
  };
  return (
    <span ref={anchor} className="tooltip-anchor" onMouseEnter={show} onMouseLeave={hide} onMouseDown={hide}>
      {children}
      {position &&
        createPortal(
          <div
            role="tooltip"
            className="tooltip"
            style={{ left: position.left, top: position.top, transform: beside ? "translateY(-50%)" : "translate(-50%, -100%)" }}
          >
            {reason}
          </div>,
          document.body,
        )}
    </span>
  );
}
