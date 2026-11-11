import type { ReactNode } from "react";
import DROP from "@material-design-icons/svg/round/arrow_drop_down.svg?raw";
import SEARCH from "@material-design-icons/svg/round/search.svg?raw";
import { ActionButtons, ActionMenu, Icon, ReasonTooltip, type ActionButton } from "../../shared/components";
import { commandKey } from "../../shared/platform";

/**
 * The bar along the top of every screen: the screen's title on the left, the field that opens the command palette
 * in the middle, and the screen's actions on the right.
 */
export function WindowToolbar({ title, onOpenPalette, actions }: { title: string; onOpenPalette: () => void; actions?: ReactNode }) {
  return (
    <>
      <header className="window-toolbar" data-tauri-drag-region>
        <span className="title-medium ellipsis toolbar-title" data-tauri-drag-region>{title}</span>
        <div className="toolbar-center" data-tauri-drag-region>
          <PaletteField onOpen={onOpenPalette} />
        </div>
        <div className="toolbar-actions" data-tauri-drag-region>{actions}</div>
      </header>
      <div className="divider" />
    </>
  );
}

/** Looks like a search field and opens the command palette, which does the searching. */
function PaletteField({ onOpen }: { onOpen: () => void }) {
  const hint = `${commandKey}K`;
  return (
    <ReasonTooltip reason={`Search and actions (${hint})`}>
      <button type="button" className="palette-field" onClick={onOpen}>
        <Icon svg={SEARCH} className="muted" />
        <span className="body-medium muted grow ellipsis" style={{ textAlign: "left" }}>Search documents, tags, actions…</span>
        <span className="label-medium muted">{hint}</span>
      </button>
    </ReasonTooltip>
  );
}

/**
 * A button with a menu of related actions beside it, such as Scan with Sync and Merge. The button waits while
 * `busyReason` names a running operation; each menu item waits only for its own lane, through its own reason.
 */
export function SplitButton({ main, menu, menuLabel, busyReason }: { main: ActionButton; menu: ActionButton[]; menuLabel: string; busyReason: string | null }) {
  return (
    <span className="row" style={{ gap: 1 }}>
      <ActionButtons buttons={[main]} enabled={busyReason === null} busyReason={busyReason} />
      <ActionMenu actions={menu} label={menuLabel} icon={DROP} />
    </span>
  );
}
