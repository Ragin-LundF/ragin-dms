import type {CSSProperties, ReactNode} from "react";
import {Icon} from "../controls/icon";

/** The card every dashboard and quick look block sits on: rounded, a hairline border and no shadow. */
export function AppCard({ children, onClick, padding, style, className }: { children: ReactNode; onClick?: () => void; padding?: number; style?: CSSProperties; className?: string }) {
  const classes = ["card", onClick ? "clickable" : "", className ?? ""].filter(Boolean).join(" ");
  const merged = padding === undefined ? style : { padding, ...style };
  if (onClick) {
    return <button type="button" className={classes} style={merged} onClick={onClick}>{children}</button>;
  }
  return <div className={classes} style={merged}>{children}</div>;
}

/** An icon on a soft tile of its own colour. */
export function IconBadge({ icon, tint, size = 40 }: { icon: string; tint: string; size?: number }) {
  return (
    <span className="icon-badge" style={{ width: size, height: size, background: `color-mix(in srgb, ${tint} 14%, transparent)`, color: tint }}>
      <Icon svg={icon} size={size * 0.55} />
    </span>
  );
}

/** The heading of a card: an icon badge, a title, an optional subtitle and actions on the right. */
export function CardTitle({ title, icon, tint, subtitle, trailing }: { title: string; icon: string; tint: string; subtitle?: string | null; trailing?: ReactNode }) {
  return (
    <div className="card-title">
      <IconBadge icon={icon} tint={tint} size={36} />
      <div className="column grow">
        <span className="title-medium nowrap">{title}</span>
        {subtitle && <span className="body-small muted ellipsis">{subtitle}</span>}
      </div>
      {trailing}
    </div>
  );
}

/** A small rounded count, as next to sidebar entries and filters. */
export function CountPill({ count, selected }: { count: number; selected?: boolean }) {
  return <span className={selected ? "count-pill selected" : "count-pill"}>{count}</span>;
}

/** A small read-only chip for a category or a tag, tinted in `color`. */
export function LabelChip({ text, color }: { text: string; color: string }) {
  return <span className="label-chip" style={{ background: `color-mix(in srgb, ${color} 14%, transparent)` }}>{text}</span>;
}

/** What an empty area shows instead of nothing: an icon, a short title and what to do next. */
export function EmptyState({ icon, title, message, action, style }: { icon: string; title: string; message: string; action?: ReactNode; style?: CSSProperties }) {
  return (
    <div className="empty-state" style={style}>
      <IconBadge icon={icon} tint="var(--primary)" size={64} />
      <span className="title-medium">{title}</span>
      <span className="body-medium message">{message}</span>
      {action}
    </div>
  );
}

/** The small upper-case title of a panel section, with optional actions on the right. */
export function SectionHeader({ title, actions }: { title: string; actions?: ReactNode }) {
  return (
    <div className="section-header">
      <span className="label-medium nowrap">{title}</span>
      <span className="grow" />
      {actions}
    </div>
  );
}

/** The large title of a screen with a muted subtitle, and the screen's actions on the right. */
export function ScreenHeader({ title, subtitle, actions }: { title: string; subtitle: string; actions?: ReactNode }) {
  return (
    <div className="row gap-m">
      <div className="column grow">
        <h1 className="headline-medium ellipsis">{title}</h1>
        <span className="body-medium muted ellipsis">{subtitle}</span>
      </div>
      {actions}
    </div>
  );
}

/** A thin progress bar; `fraction` null shows an indeterminate one. */
export function ProgressBar({ fraction, width }: { fraction: number | null; width?: number | string }) {
  return (
    <div className={fraction === null ? "progress indeterminate" : "progress"} style={{ width }} role="progressbar" aria-valuenow={fraction === null ? undefined : Math.round(fraction * 100)}>
      <div className="bar" style={fraction === null ? undefined : { width: `${Math.min(1, Math.max(0, fraction)) * 100}%` }} />
    </div>
  );
}

export function Spinner({ small }: { small?: boolean }) {
  return <div className={small ? "spinner small" : "spinner"} role="progressbar" />;
}
