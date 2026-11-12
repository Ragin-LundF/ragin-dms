import COLLAPSE from "@material-design-icons/svg/round/keyboard_double_arrow_left.svg?raw";
import EXPAND from "@material-design-icons/svg/round/keyboard_double_arrow_right.svg?raw";
import logo from "../../assets/app-icon.png";
import {Icon, ReasonTooltip} from "../../shared/components";
import {NAV_GROUPS, type Screen, SCREENS} from "../screens";

export type NavigationCounts = { inbox: number; tasks: number; toPay: number; cases: number };

/**
 * The app's left column: the brand, the screens in their groups with the inbox, task, overdue bill and due follow-up
 * counts, and Settings at
 * the bottom. While `collapsed` it is a rail of icons that name themselves on hover. `hidden` screens are left
 * out, such as Ask while no AI is set up.
 */
export function NavigationSidebar({ screen, hidden, counts, collapsed, onToggleCollapsed, onScreenChange }: {
  screen: Screen;
  hidden: Screen[];
  counts: NavigationCounts;
  collapsed: boolean;
  onToggleCollapsed: () => void;
  onScreenChange: (screen: Screen) => void;
}) {
  const item = (entry: (typeof SCREENS)[number]) => (
    <NavItem key={entry.id} label={entry.label} icon={entry.icon} selected={entry.id === screen} compact={collapsed} badge={badgeOf(entry.id, counts)} onClick={() => onScreenChange(entry.id)} />
  );
  return (
    <nav className={collapsed ? "navigation collapsed" : "navigation"} data-tauri-drag-region>
      <div className="navigation-inner" data-tauri-drag-region>
        <BrandMark compact={collapsed} />
        {NAV_GROUPS.map((group) => (
          <div key={group.id} className="column" style={{ gap: 1 }}>
            {collapsed ? <div style={{ height: "var(--space-m)" }} /> : <span className="label-small muted nav-group">{group.label.toUpperCase()}</span>}
            {SCREENS.filter((entry) => entry.group === group.id && !hidden.includes(entry.id)).map(item)}
          </div>
        ))}
        <div className="grow" data-tauri-drag-region />
        {SCREENS.filter((entry) => entry.group === null).map(item)}
        <NavCollapseButton collapsed={collapsed} onToggle={onToggleCollapsed} />
      </div>
    </nav>
  );
}

type Badge = { count: number; warning: boolean };

const BADGES: Partial<Record<Screen, (counts: NavigationCounts) => Badge>> = {
  inbox: (counts) => ({ count: counts.inbox, warning: false }),
  tasks: (counts) => ({ count: counts.tasks, warning: true }),
  bills: (counts) => ({ count: counts.toPay, warning: true }),
  cases: (counts) => ({ count: counts.cases, warning: true }),
};

/** The badge of a screen; null for screens without one or a count of 0. */
function badgeOf(screen: Screen, counts: NavigationCounts): Badge | null {
  const badge = BADGES[screen]?.(counts) ?? null;
  return badge && badge.count > 0 ? badge : null;
}

/**
 * One screen in the navigation; the selected one sits on a surface pill with a primary icon. A compact item shows
 * only its icon, names the screen on hover, and shows its badge as a dot.
 */
function NavItem({ label, icon, selected, compact, badge, onClick }: { label: string; icon: string; selected: boolean; compact: boolean; badge: Badge | null; onClick: () => void }) {
  const color = badge?.warning ? "var(--status-warning)" : "var(--primary)";
  const hover = [label, badge ? String(badge.count) : null].filter(Boolean).join(": ");
  const row = (
    <button type="button" className={selected ? "nav-item selected" : "nav-item"} aria-current={selected ? "page" : undefined} aria-label={compact ? hover : undefined} onClick={onClick}>
      <span className="nav-icon">
        <Icon svg={icon} />
        {compact && badge && <span className="nav-dot" style={{ background: color }} />}
      </span>
      {!compact && (
        <>
          <span className="label-large grow ellipsis nav-label">{label}</span>
          {badge && <span className="label-small nav-badge" style={{ color, background: `color-mix(in srgb, ${color} 14%, transparent)` }}>{badge.count}</span>}
        </>
      )}
    </button>
  );
  return compact ? <ReasonTooltip reason={hover} beside>{row}</ReasonTooltip> : row;
}

/** Switches the navigation between icons only and the full view with labels. */
function NavCollapseButton({ collapsed, onToggle }: { collapsed: boolean; onToggle: () => void }) {
  const label = collapsed ? "Expand menu" : "Collapse menu";
  const row = (
    <button type="button" className="nav-item nav-collapse" aria-label={label} onClick={onToggle}>
      <Icon svg={collapsed ? EXPAND : COLLAPSE} className="muted" />
      {!collapsed && <span className="label-large muted grow">{label}</span>}
    </button>
  );
  return collapsed ? <ReasonTooltip reason={label} beside>{row}</ReasonTooltip> : row;
}

/** The logo and the app's name; a compact mark is the logo alone. */
function BrandMark({ compact }: { compact: boolean }) {
  return (
    <div className="row gap-m" data-tauri-drag-region>
      <img src={logo} alt="Ragin-DMS logo" width={compact ? 40 : 44} height={compact ? 40 : 44} draggable={false} />
      {!compact && (
        <div className="column">
          <span className="title-medium">Ragin-DMS</span>
          <span className="label-small muted">Your documents</span>
        </div>
      )}
    </div>
  );
}
