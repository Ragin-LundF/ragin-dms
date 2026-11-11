import ARROW_DOWN from "@material-design-icons/svg/round/keyboard_arrow_down.svg?raw";
import ARROW_RIGHT from "@material-design-icons/svg/round/keyboard_arrow_right.svg?raw";
import type {ReactNode} from "react";
import {Icon, ReasonTooltip} from "../../../shared/components";

/**
 * One view of the sidebar with its icon and count (Kotlin `SidebarItem`). A `warning` count above zero is a badge in
 * the warning tint; in the folded rail only the icon shows, the badge becomes a dot and a tooltip names the view.
 */
export function SidebarItem({ label, icon, count, selected, folded, warning = false, menu, onClick }: {
  label: string;
  icon: string;
  count: number | null;
  selected: boolean;
  folded: boolean;
  warning?: boolean;
  menu?: ReactNode;
  onClick: () => void;
}) {
  const badge = warning && (count ?? 0) > 0;
  const item = (
    <div
      role="button"
      tabIndex={-1}
      aria-label={folded ? label : undefined}
      aria-current={selected || undefined}
      className={["lib-item", selected ? "selected" : "", folded ? "folded" : ""].filter(Boolean).join(" ")}
      onClick={onClick}
    >
      <span className="lib-item-icon">
        <Icon svg={icon} />
        {folded && badge && <span className="lib-item-dot" />}
      </span>
      {!folded && (
        <>
          <span className="lib-item-label body-medium ellipsis">{label}</span>
          {count !== null && <span className={badge ? "lib-count badge label-small" : "lib-count label-small"}>{count}</span>}
          {menu && <span onClick={(event) => event.stopPropagation()}>{menu}</span>}
        </>
      )}
    </div>
  );
  return folded ? <ReasonTooltip reason={label} beside>{item}</ReasonTooltip> : item;
}

/**
 * The heading of a sidebar section (Kotlin `SidebarSection`); a click folds or unfolds it. In the rail it is a divider.
 */
export function SidebarSection({ title, expanded, folded, onToggle, add }: {
  title: string;
  expanded: boolean;
  folded: boolean;
  onToggle: () => void;
  add?: ReactNode;
}) {
  if (folded) return <div className="lib-rail-divider" role="separator" />;
  return (
    <div
      role="button"
      tabIndex={-1}
      className="lib-section"
      aria-label={expanded ? `Fold ${title}` : `Unfold ${title}`}
      aria-expanded={expanded}
      onClick={onToggle}
    >
      <span className="label-small grow">{title.toUpperCase()}</span>
      {add && <span onClick={(event) => event.stopPropagation()}>{add}</span>}
      <Icon svg={expanded ? ARROW_DOWN : ARROW_RIGHT} />
    </div>
  );
}
