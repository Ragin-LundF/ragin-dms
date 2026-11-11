import {type ReactNode, useEffect, useLayoutEffect, useRef, useState} from "react";
import {createPortal} from "react-dom";
import MORE from "@material-design-icons/svg/round/more_vert.svg?raw";
import type {ActionButton} from "./action";
import {IconButton} from "./buttons";
import {Icon} from "./icon";
import {ReasonTooltip} from "./tooltip";

/**
 * A dropdown menu below `anchor` (or at a point, for context menus); closes on Escape, on a click outside and
 * after a choice. Keeps itself inside the window.
 */
export function Menu({ anchor, onClose, children, align = "start" }: { anchor: DOMRect | { x: number; y: number }; onClose: () => void; children: ReactNode; align?: "start" | "end" }) {
  const ref = useRef<HTMLDivElement>(null);
  const [style, setStyle] = useState<{ left: number; top: number; visibility: "hidden" | "visible" }>({ left: 0, top: 0, visibility: "hidden" });
  useLayoutEffect(() => {
    const menu = ref.current?.getBoundingClientRect();
    if (!menu) return;
    const isRect = "bottom" in anchor;
    let left = isRect ? (align === "end" ? anchor.right - menu.width : anchor.left) : anchor.x;
    let top = isRect ? anchor.bottom + 2 : anchor.y;
    if (left + menu.width > window.innerWidth - 8) left = window.innerWidth - menu.width - 8;
    if (top + menu.height > window.innerHeight - 8) top = Math.max(8, (isRect ? anchor.top - menu.height - 2 : window.innerHeight - menu.height - 8));
    setStyle({ left: Math.max(8, left), top, visibility: "visible" });
  }, [anchor, align]);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        onClose();
      }
    };
    const onDown = (event: MouseEvent) => {
      if (!ref.current?.contains(event.target as Node)) onClose();
    };
    window.addEventListener("keydown", onKey, true);
    window.addEventListener("mousedown", onDown, true);
    return () => {
      window.removeEventListener("keydown", onKey, true);
      window.removeEventListener("mousedown", onDown, true);
    };
  }, [onClose]);
  return createPortal(
    <div ref={ref} role="menu" className="menu" style={style} onClick={(event) => event.stopPropagation()}>
      {children}
    </div>,
    document.body,
  );
}

/** One entry of a [`Menu`]; a disabled one explains why in its tooltip. */
export function MenuItem({ label, icon, shortcut, disabledReason, disabled, onClick, trailing }: {
  label: ReactNode;
  icon?: string;
  shortcut?: string;
  disabledReason?: string | null;
  disabled?: boolean;
  trailing?: ReactNode;
  onClick: () => void;
}) {
  const item = (
    <button type="button" role="menuitem" className="menu-item" disabled={disabled || !!disabledReason} onClick={onClick}>
      {icon && <Icon svg={icon} />}
      <span className="ellipsis">{label}</span>
      {shortcut && <span className="shortcut">{shortcut}</span>}
      {trailing}
    </button>
  );
  return disabledReason ? <ReasonTooltip reason={disabledReason}><div style={{ width: "100%" }}>{item}</div></ReasonTooltip> : item;
}

export function MenuDivider() {
  return <div className="menu-divider" role="separator" />;
}

/** Opens a menu below the element that was clicked. */
export function useMenuAnchor() {
  const [anchor, setAnchor] = useState<DOMRect | null>(null);
  return {
    anchor,
    open: (event: { currentTarget: Element }) => setAnchor(event.currentTarget.getBoundingClientRect()),
    close: () => setAnchor(null),
  };
}

/** A 3-dot button that opens `actions` as a menu; a disabled action explains in its tooltip why it cannot be used. */
export function ActionMenu({ actions, label, enabled = true, busyReason, icon = MORE }: { actions: ActionButton[]; label: string; enabled?: boolean; busyReason?: string | null; icon?: string }) {
  const menu = useMenuAnchor();
  return (
    <>
      <IconButton icon={icon} label={label} small onClick={menu.open} />
      {menu.anchor && (
        <Menu anchor={menu.anchor} onClose={menu.close}>
          {actions.map((action) => (
            <MenuItem
              key={action.label}
              label={action.label}
              icon={action.icon}
              shortcut={action.shortcut}
              disabledReason={action.disabledReason ?? (enabled ? null : busyReason)}
              onClick={() => {
                menu.close();
                action.onClick();
              }}
            />
          ))}
        </Menu>
      )}
    </>
  );
}
