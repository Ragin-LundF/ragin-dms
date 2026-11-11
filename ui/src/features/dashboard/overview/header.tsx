import {useState} from "react";
import AUTO_AWESOME from "@material-design-icons/svg/round/auto_awesome.svg?raw";
import type {AiBandDto_Serialize, StatColumnDto_Serialize, StatLink} from "../../../ipc";
import {busyIn, useSession} from "../../../session/session";
import {AppCard, Button, Icon, ReasonTooltip, waitFor} from "../../../shared/components";
import {formatCount} from "../../../shared/format/sizes";
import {aiBatches} from "../ai-batches";
import {Sparkline} from "../charts/sparkline";
import {useGrowIn} from "../charts/grow-in";

const NOON = 12;
const EVENING = 18;

function greetingAt(hour: number): string {
  if (hour < NOON) return "Good morning";
  if (hour < EVENING) return "Good afternoon";
  return "Good evening";
}

/** A greeting by the time of day, computed once, and the size of the library (Kotlin `DashboardHeader`). */
export function DashboardHeader({ line }: { line: string }) {
  const [greeting] = useState(() => greetingAt(new Date().getHours()));
  return (
    <div className="dashboard-header">
      <span className="headline-small nowrap">{greeting}</span>
      <span className="body-medium muted ellipsis">{line}</span>
    </div>
  );
}

/** The headline figures in one card between hairlines; a figure with a link opens it (Kotlin `StatRow`). */
export function StatRow({ columns, onOpen }: { columns: StatColumnDto_Serialize[]; onOpen: (link: StatLink) => void }) {
  return (
    <AppCard padding={0} className="stat-row">
      {columns.map((column, index) => (
        <div key={column.label} className="stat-cell">
          {index > 0 && <div className="vdivider" />}
          <StatColumn column={column} onClick={column.link ? () => onOpen(column.link as StatLink) : undefined} />
        </div>
      ))}
    </AppCard>
  );
}

function StatColumn({ column, onClick }: { column: StatColumnDto_Serialize; onClick?: () => void }) {
  const meter = useGrowIn(column.meter ?? 0);
  const content = (
    <>
      <span className="label-medium muted nowrap">{column.label}</span>
      <span className="row gap-m">
        <span className="headline-medium nowrap">{column.count === null ? column.value : formatCount(column.count)}</span>
        {column.trend.length > 1 && <Sparkline values={column.trend} />}
      </span>
      {column.meter !== null && (
        <span className="stat-meter">
          <span style={{ width: `${meter * 100}%` }} />
        </span>
      )}
      <span className="body-small muted ellipsis">{onClick ? `${column.caption} →` : column.caption}</span>
    </>
  );
  if (!onClick) return <div className="stat-column">{content}</div>;
  return (
    <button type="button" className="stat-column clickable" onClick={onClick}>
      {content}
    </button>
  );
}

/** What the AI can do now, framed in the AI gradient; nothing changes before the review (Kotlin `AiBand`). */
export function AiBand({ band }: { band: AiBandDto_Serialize }) {
  const busy = busyIn(useSession(), "ai");
  const reason = busy ? waitFor(busy) : null;
  return (
    <div className="ai-band">
      <span className="ai-sparkle">
        <Icon svg={AUTO_AWESOME} />
      </span>
      <div className="column grow">
        <span className="title-small">{band.headline}</span>
        <span className="body-small muted">The AI makes suggestions. Nothing changes until you approve each one.</span>
      </div>
      {band.classifyLabel && (
        <ReasonTooltip reason={reason}>
          <Button variant="outlined" disabled={reason !== null} onClick={aiBatches.classifyAll}>
            {band.classifyLabel}
          </Button>
        </ReasonTooltip>
      )}
      {band.tagLabel && (
        <ReasonTooltip reason={reason}>
          <button type="button" className="btn ai-gradient-button has-icon" disabled={reason !== null} onClick={aiBatches.tagAll}>
            <Icon svg={AUTO_AWESOME} />
            {band.tagLabel}
          </button>
        </ReasonTooltip>
      )}
    </div>
  );
}
