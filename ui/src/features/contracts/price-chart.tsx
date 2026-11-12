import type {ContractHistoryPointDto} from "../../ipc";

const WIDTH = 420;
const HEIGHT = 150;
const LEFT = 30;
const RIGHT = 20;
const BOTTOM = 20;
const TOP = 20;

/** The chart colour of a contract: the fixed order, then the neutral one. */
export function chartColour(index: number): string {
  return index < 4 ? `var(--chart-${index + 1})` : "var(--chart-neutral)";
}

/**
 * The monthly price over time: a thin line in the contract's colour, each price as a labelled point, a rise marked
 * with an arrow in its label and the warning colour.
 */
export function PriceChart({ points, colour }: { points: ContractHistoryPointDto[]; colour: string }) {
  if (points.length === 0) return <p className="body-small muted">No price history yet</p>;
  const max = Math.max(...points.map((point) => point.cents)) * 1.15 || 1;
  const x = (index: number) => LEFT + (index * (WIDTH - LEFT - RIGHT)) / Math.max(points.length - 1, 1);
  const y = (cents: number) => HEIGHT - BOTTOM - (cents / max) * (HEIGHT - BOTTOM - TOP);
  return (
    <svg className="price-chart" viewBox={`0 0 ${WIDTH} ${HEIGHT}`} role="img" aria-label="Monthly price over time">
      <line className="price-chart-axis" x1={LEFT} x2={WIDTH - RIGHT} y1={HEIGHT - BOTTOM} y2={HEIGHT - BOTTOM} />
      <polyline points={points.map((point, index) => `${x(index)},${y(point.cents)}`).join(" ")} fill="none" stroke={colour} strokeWidth={1.5} />
      {points.map((point, index) => (
        <g key={point.validFrom}>
          <circle cx={x(index)} cy={y(point.cents)} r={4} fill={point.rose ? "var(--status-warning)" : colour} />
          <text className="price-chart-value" x={x(index)} y={y(point.cents) - 10} textAnchor="middle">
            {point.rose ? `↑ ${point.label}` : point.label}
          </text>
          <text className="price-chart-day" x={x(index)} y={HEIGHT - 5} textAnchor="middle">
            {point.validFrom.slice(0, 7)}
          </text>
        </g>
      ))}
    </svg>
  );
}
