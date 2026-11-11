import {useGrowIn} from "./grow-in";

export type DonutSlice = { label: string; value: number; color: string };

const DIAMETER = 148;
const STROKE = 18;
const GAP_DEGREES = 2;

/** A donut with the total in its centre and a legend with count and share of every slice (Kotlin `DonutChart`). */
export function DonutChart({ slices, centerValue, centerLabel }: { slices: DonutSlice[]; centerValue: string; centerLabel: string }) {
  const total = Math.max(1, slices.reduce((sum, slice) => sum + slice.value, 0));
  const progress = useGrowIn(1);
  const radius = (DIAMETER - STROKE) / 2;
  const circumference = 2 * Math.PI * radius;
  const visible = slices.filter((slice) => slice.value > 0);
  const gap = visible.length > 1 ? (GAP_DEGREES / 360) * circumference : 0;
  let start = 0;
  return (
    <div className="donut">
      <div className="donut-ring" style={{ width: DIAMETER, height: DIAMETER }}>
        <svg width={DIAMETER} height={DIAMETER} viewBox={`0 0 ${DIAMETER} ${DIAMETER}`} aria-hidden>
          <circle cx={DIAMETER / 2} cy={DIAMETER / 2} r={radius} fill="none" stroke="var(--surface-high)" strokeWidth={STROKE} />
          {visible.map((slice) => {
            const sweep = (slice.value / total) * circumference * progress;
            const offset = start;
            start += sweep;
            return (
              <circle
                key={slice.label}
                className="donut-slice"
                cx={DIAMETER / 2}
                cy={DIAMETER / 2}
                r={radius}
                fill="none"
                stroke={slice.color}
                strokeWidth={STROKE}
                strokeDasharray={`${Math.max(0, sweep - gap)} ${circumference}`}
                strokeDashoffset={-(offset + gap / 2)}
                transform={`rotate(-90 ${DIAMETER / 2} ${DIAMETER / 2})`}
              />
            );
          })}
        </svg>
        <div className="donut-center">
          <span className="headline-small">{centerValue}</span>
          <span className="label-small muted">{centerLabel}</span>
        </div>
      </div>
      <div className="column gap-s">
        {slices.map((slice) => (
          <div key={slice.label} className="row legend-row">
            <span className="legend-dot" style={{ background: slice.color }} />
            <span className="body-medium nowrap legend-label">{slice.label}</span>
            <span className="label-large">{slice.value}</span>
            <span className="body-small muted">{Math.round((100 * slice.value) / total)} %</span>
          </div>
        ))}
      </div>
    </div>
  );
}
