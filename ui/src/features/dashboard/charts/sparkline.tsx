/** A small line without axes, a soft area below it and a dot on the latest value; needs two values (Kotlin `Sparkline`). */
export function Sparkline({ values, width = 64, height = 24 }: { values: number[]; width?: number; height?: number }) {
  if (values.length < 2) return null;
  const dot = 2.2;
  const lowest = Math.min(...values);
  const range = Math.max(1, Math.max(...values) - lowest);
  const points = values.map((value, index) => ({
    x: dot + ((width - dot * 2) * index) / (values.length - 1),
    y: dot + (height - dot * 2) * (1 - (value - lowest) / range),
  }));
  const line = points.map(({ x, y }, index) => `${index === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`).join(" ");
  const first = points[0] ?? { x: 0, y: 0 };
  const last = points[points.length - 1] ?? first;
  const area = `${line} L${last.x.toFixed(1)},${height} L${first.x.toFixed(1)},${height} Z`;
  return (
    <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} className="sparkline" aria-hidden>
      <path d={area} fill="var(--primary)" fillOpacity={0.12} />
      <path d={line} fill="none" stroke="var(--primary)" strokeWidth={1.6} />
      <circle cx={last.x} cy={last.y} r={dot} fill="var(--primary)" />
    </svg>
  );
}
