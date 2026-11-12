import {useQuery} from "@tanstack/react-query";
import {commands, unwrap} from "../../../ipc";
import {LIBRARY_QUERY} from "../../../session/session";
import {AppCard} from "../../../shared/components";
import {relativeInstant} from "../../../shared/format/dates";
import {goTo} from "../../../shell/screens";

const TONE_COLOR = { good: "var(--status-good)", warning: "var(--status-warning)", error: "var(--error)", neutral: "var(--on-surface)" } as const;

/** The status card "Archive health" on the dashboard; a click opens Settings › Archive health. */
export function HealthCard() {
  const state = useQuery({ queryKey: [LIBRARY_QUERY, "health"], queryFn: () => unwrap(commands.healthState()) }).data;
  if (!state) return null;
  return (
    <AppCard className="status-card" onClick={() => goTo("settings")}>
      <span className="body-small muted">Archive health</span>
      <span className="title-large" style={{ color: TONE_COLOR[state.tone] }}>{state.headline}</span>
      <span className="body-small muted">{state.lastCheckAt ? `Last check: ${relativeInstant(state.lastCheckAt)}` : "Not checked on this device yet"}</span>
    </AppCard>
  );
}
