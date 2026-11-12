import type {HealthToneDto} from "../../../ipc";

/** The colour of a state; the label beside it always names the state too. */
export const TONE_COLOR: Record<HealthToneDto, string> = {
  good: "var(--status-good)",
  warning: "var(--status-warning)",
  error: "var(--error)",
  neutral: "var(--outline)",
};
