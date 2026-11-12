import type {HealthStateDto_Serialize} from "../../../ipc";
import {TONE_COLOR} from "./tone";

const RADIUS = 50;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

/** The ring with the number of files checked; its arc is the share of the pass done. */
export function HealthRing({ state }: { state: HealthStateDto_Serialize }) {
  const offset = CIRCUMFERENCE * (1 - (state.done ?? 0));
  return (
    <svg className="health-ring" viewBox="0 0 120 120" role="img" aria-label={`${state.filesChecked} files checked`}>
      <circle cx="60" cy="60" r={RADIUS} fill="none" stroke="var(--surface-high)" strokeWidth="10" />
      <circle
        cx="60" cy="60" r={RADIUS} fill="none" stroke={TONE_COLOR[state.tone]} strokeWidth="10" strokeLinecap="round"
        strokeDasharray={CIRCUMFERENCE} strokeDashoffset={offset} transform="rotate(-90 60 60)"
      />
      <text x="60" y="58" textAnchor="middle" className="health-ring-count">{state.filesChecked.toLocaleString()}</text>
      <text x="60" y="76" textAnchor="middle" className="health-ring-label">files checked</text>
    </svg>
  );
}
