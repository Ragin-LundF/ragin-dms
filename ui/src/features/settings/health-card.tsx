import HEALTH from "@material-design-icons/svg/round/health_and_safety.svg?raw";
import {useQuery} from "@tanstack/react-query";
import {commands, type HealthScheduleDto, unwrap} from "../../ipc";
import {LIBRARY_QUERY, reportError} from "../../session/session";
import {AppCard, Button, CardTitle, Chip, ReasonTooltip} from "../../shared/components";
import {absoluteDate, relativeInstant, timeOfDay} from "../../shared/format/dates";
import {HealthFindings} from "./health/health-findings";
import {HealthFolders} from "./health/health-folders";
import {HealthRing} from "./health/health-ring";
import {HealthSnapshots} from "./health/health-snapshots";

const SCHEDULES: { value: HealthScheduleDto; label: string }[] = [
  { value: "off", label: "Off" },
  { value: "weekly", label: "Weekly" },
  { value: "monthly", label: "Monthly" },
];

export function useHealthState() {
  return useQuery({ queryKey: [LIBRARY_QUERY, "health"], queryFn: () => unwrap(commands.healthState()) });
}

/** Settings › Archive health: the check of every copy, its findings and the catalog snapshots. */
export function HealthSettingsCard({ busyReason }: { busyReason: string | null }) {
  const state = useHealthState().data;
  return (
    <AppCard className="settings-card">
      <CardTitle title="Archive health" icon={HEALTH} tint="var(--secondary)" subtitle="Every file in every library folder read again and compared with its stored SHA-256" />
      {!state ? (
        <span className="body-medium muted">Open a library to check it.</span>
      ) : (
        <div className="health-layout">
          <div className="column gap-s health-summary">
            <HealthRing state={state} />
            <span className="title-small">{state.headline}</span>
            <span className="body-small muted" title={state.lastCheckAt ? absoluteDate(state.lastCheckAt) : undefined}>
              {state.lastCheckAt ? `Last check: ${relativeInstant(state.lastCheckAt)} ${timeOfDay(state.lastCheckAt)}` : "Never checked on this device"}
            </span>
            <div className="row gap-xs">
              {SCHEDULES.map((schedule) => (
                <Chip
                  key={schedule.value} label={schedule.label} selected={state.schedule === schedule.value} disabled={busyReason !== null}
                  onClick={() => void unwrap(commands.healthSetSchedule(schedule.value)).catch(reportError)}
                />
              ))}
            </div>
            <span className="body-small muted">
              {state.schedule === "off" ? "Runs only when you start it." : "Starts in the background when the library opens and a check is due."}
            </span>
            <ReasonTooltip reason={busyReason}>
              <Button variant="filled" disabled={busyReason !== null} onClick={() => void unwrap(commands.healthCheckNow()).catch(reportError)}>
                {state.unfinished ? "Continue check" : "Check now"}
              </Button>
            </ReasonTooltip>
          </div>
          <div className="column gap-l grow">
            <HealthFolders folders={state.folders} />
            <HealthFindings findings={state.findings} />
            <HealthSnapshots snapshots={state.snapshots} busyReason={busyReason} />
          </div>
        </div>
      )}
    </AppCard>
  );
}
