import type {ChartPointDto} from "../../../ipc";
import {useGrowIn} from "./grow-in";

/**
 * Vertical bars on a common baseline; the last and the hovered bar show their value, every bar is labelled below.
 * A click on a bar opens what it counts (Kotlin `ColumnChart`, `ColumnBar`).
 */
export function ColumnChart({ points, color, onClick }: { points: ChartPointDto[]; color: string; onClick: (point: ChartPointDto) => void }) {
  const max = Math.max(1, ...points.map((point) => point.value));
  return (
    <div className="column-chart">
      <div className="column-bars">
        {points.map((point, index) => (
          <ColumnBar key={point.id} point={point} fraction={point.value / max} color={color} labelled={index === points.length - 1} onClick={() => onClick(point)} />
        ))}
      </div>
      <div className="column-baseline" />
      <div className="column-labels">
        {points.map((point) => (
          <span key={point.id} className="label-small muted nowrap">{point.label}</span>
        ))}
      </div>
    </div>
  );
}

function ColumnBar({ point, fraction, color, labelled, onClick }: { point: ChartPointDto; fraction: number; color: string; labelled: boolean; onClick: () => void }) {
  const grown = useGrowIn(fraction);
  return (
    <button type="button" className={labelled ? "column-bar labelled" : "column-bar"} onClick={onClick} aria-label={`${point.label}: ${point.value}`}>
      <span className="label-medium column-value">{point.value}</span>
      <span className="column-fill" style={{ height: `${Math.max(grown, 0.01) * 100}%`, background: color }} />
    </button>
  );
}
