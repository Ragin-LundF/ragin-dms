import CLOUD_OFF from "@material-design-icons/svg/round/cloud_off.svg?raw";
import REFRESH from "@material-design-icons/svg/round/refresh.svg?raw";
import SETTINGS from "@material-design-icons/svg/round/settings.svg?raw";
import type {SessionState} from "../session/session";
import {Button, EmptyState} from "../shared/components";

/** Why the library could not be opened, with the ways to fix it. */
export function LibraryProblem({ status, onRetry, onOpenSettings }: { status: SessionState["status"]; onRetry: () => void; onOpenSettings: () => void }) {
  let text = "The library is not open.";
  if (status.kind === "primaryFailed") {
    const cause = [status.reason, status.detail].filter(Boolean).join(": ");
    text = `The primary folder or its catalog cannot be read (${cause}). Connect the drive and try again, or make another folder the primary in the settings.`;
  } else if (status.kind === "failed") {
    text = `The library could not be opened: ${status.detail ?? "unknown error"}`;
  }
  return (
    <EmptyState
      icon={CLOUD_OFF}
      title="The library is not open"
      message={text}
      style={{ height: "100%" }}
      action={
        <div className="row gap-s">
          <Button variant="filled" icon={REFRESH} onClick={onRetry}>Try again</Button>
          <Button icon={SETTINGS} onClick={onOpenSettings}>Settings</Button>
        </div>
      }
    />
  );
}
