import type {AttentionDto_Serialize, AttentionRowDto_Serialize, AttentionSeverity} from "../../../ipc";
import {AppCard, Button, CountPill, IconBadge} from "../../../shared/components";
import {runButton} from "../actions";
import {DashButtons, DuplicateCopies, firstOpen, RowThumbnail} from "../tasks/buttons";
import {iconOf} from "../tasks/icons";

const STRIPE: Record<AttentionSeverity, string> = {
  passed: "var(--error)",
  soon: "var(--status-warning)",
  info: "var(--primary)",
};

/** "Needs your attention": the most urgent rows, how many there are, and a way to every task (Kotlin `AttentionPanel`). */
export function AttentionPanel({ attention, onShowAllTasks }: { attention: AttentionDto_Serialize; onShowAllTasks: () => void }) {
  return (
    <AppCard padding={0} className="attention-panel">
      <div className="attention-header">
        <span className="title-medium nowrap">Needs your attention</span>
        {attention.total > 0 && <CountPill count={attention.total} />}
        <div className="row wrap gap-s grow attention-header-buttons">
          <DashButtons buttons={attention.headerButtons} inline />
          <Button variant="text" onClick={onShowAllTasks}>All tasks</Button>
        </div>
      </div>
      {attention.rows.length === 0 && (
        <>
          <div className="divider" />
          <span className="body-medium muted attention-empty">
            Nothing needs you right now. New, missing and changed files appear here after a scan.
          </span>
        </>
      )}
      {attention.rows.map((row) => (
        <div key={row.key}>
          <div className="divider" />
          <AttentionRow row={row} />
        </div>
      ))}
    </AppCard>
  );
}

/** A stripe in the severity's colour, a thumbnail, title and detail, and the row's buttons, open buttons first. */
function AttentionRow({ row }: { row: AttentionRowDto_Serialize }) {
  const open = firstOpen(row.opens, row.copies);
  return (
    <div className="attention-row" style={{ "--stripe": STRIPE[row.severity] } as React.CSSProperties}>
      <RowThumbnail
        file={row.preview}
        onClick={() => open && void runButton(open)}
        fallback={<IconBadge icon={iconOf(row.icon) ?? ""} tint="var(--on-surface-variant)" size={32} />}
      />
      <div className="column attention-text">
        <span className="title-small ellipsis">{row.title}</span>
        <span className="body-small muted ellipsis">{row.detail}</span>
        <DuplicateCopies copies={row.copies} />
      </div>
      <DashButtons buttons={[...row.opens, ...row.buttons]} className="attention-buttons" />
    </div>
  );
}
