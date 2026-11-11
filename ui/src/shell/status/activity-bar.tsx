import CLOSE from "@material-design-icons/svg/round/close.svg?raw";
import TERMINAL from "@material-design-icons/svg/round/terminal.svg?raw";
import {commands} from "../../ipc";
import {type OperationProgress, useProgress, useSession} from "../../session/session";
import {Icon, ProgressBar} from "../../shared/components";
import {formatCount} from "../../shared/format/sizes";
import {libraryStatusText} from "./status-texts";

/**
 * The one status bar along the bottom of the window: on the left the state of the library and the last scan, on
 * the right every running operation with its progress, Cancel where stopping is safe, and the log's switch.
 */
export function ActivityBar({ logVisible, onToggleLog }: { logVisible: boolean; onToggleLog: () => void }) {
  const state = useSession();
  const progress = useProgress();
  const dot = { open: "var(--status-good)", loading: "var(--status-warning)" }[state.status.kind as string] ?? "var(--error)";
  return (
    <footer className="activity-bar">
      <span className="status-dot" style={{ background: dot }} />
      <span className="label-small muted ellipsis grow">{libraryStatusText(state)}</span>
      {state.operations.map((operation) => (
        <span key={operation.lane} className="row gap-l">
          <ProgressLine busy={operation.label} progress={progress[operation.lane] ?? null} />
          {operation.cancellable && (
            <button type="button" className="status-button" onClick={() => void commands.cancelOperation(operation.lane)}>
              <Icon svg={CLOSE} size={12} />
              <span className="label-small muted">Cancel</span>
            </button>
          )}
        </span>
      ))}
      <button type="button" className="status-button" onClick={onToggleLog}>
        <Icon svg={TERMINAL} size={12} className={logVisible ? "primary-text" : "muted"} />
        <span className="label-small muted">{logVisible ? "Hide log" : "Log"}</span>
      </button>
    </footer>
  );
}

/** A running operation in one short line: its step, its counts and a small progress bar. */
export function ProgressLine({ busy, progress }: { busy: string; progress: OperationProgress | null }) {
  const detail = progress?.detail ? ` · ${progress.detail}` : "";
  const total = progress?.total ?? null;
  const fraction = progress && total !== null ? (total === 0 ? 1 : progress.done / total) : null;
  return (
    <span className="row gap-s">
      <span className="label-small ellipsis" style={{ maxWidth: "var(--status-text-max-width)" }}>{(progress?.step ?? busy) + detail}</span>
      {progress && total !== null && (
        <span className="label-small primary-text nowrap">{formatCount(progress.done)} of {formatCount(total)}</span>
      )}
      <ProgressBar fraction={fraction} width="var(--status-progress-width)" />
    </span>
  );
}
