import type {ChartPointDto} from "../../../ipc";
import {useGrowIn} from "./grow-in";

/** A ranked list: each entry with its count and a thin bar of its size; a click opens the entry (Kotlin `BarList`). */
export function BarList({ points, onClick }: { points: ChartPointDto[]; onClick: (point: ChartPointDto) => void }) {
  const max = Math.max(1, ...points.map((point) => point.value));
  return (
    <div className="bar-list scroll">
      {points.map((point) => (
        <BarEntry key={point.id} point={point} fraction={point.value / max} onClick={() => onClick(point)} />
      ))}
    </div>
  );
}

function BarEntry({ point, fraction, onClick }: { point: ChartPointDto; fraction: number; onClick: () => void }) {
  const grown = useGrowIn(fraction);
  return (
    <button type="button" className="bar-entry" onClick={onClick}>
      <span className="row">
        <span className="body-medium ellipsis grow">{point.label}</span>
        <span className="label-medium muted">{point.value}</span>
      </span>
      <span className="bar-track">
        <span className="bar-fill" style={{ width: `${Math.max(grown, 0.02) * 100}%` }} />
      </span>
    </button>
  );
}
