import EVENT from "@material-design-icons/svg/round/event.svg?raw";
import TRAVEL_EXPLORE from "@material-design-icons/svg/round/travel_explore.svg?raw";
import { commands, unwrap } from "../../ipc";
import { operations } from "../../session/operations";
import { reportError } from "../../session/session";
import { ActionButtons, AppCard, CardTitle, ReasonTooltip, Switch } from "../../shared/components";

/** How a scan treats files the catalog does not know yet, and the dates of documents that have none. */
export function ScanningCard({ addNewFiles, busyReason }: { addNewFiles: boolean; busyReason: string | null }) {
  return (
    <AppCard className="settings-card">
      <CardTitle title="Scanning" icon={TRAVEL_EXPLORE} tint="var(--tertiary)" subtitle="What happens with files the library does not know yet" />
      <div className="row gap-l">
        <div className="column grow">
          <span className="title-small">Add new files while scanning</span>
          <span className="body-small muted">
            {addNewFiles
              ? "New files appear in the library as soon as the scan has read them. Files stay where they are; only the catalog learns about them."
              : "New files are listed on the dashboard, and you adopt them one by one or all at once."}
          </span>
        </div>
        <ReasonTooltip reason={busyReason}>
          <Switch
            label="Add new files while scanning"
            checked={addNewFiles}
            disabled={busyReason !== null}
            onChange={(enabled) => void unwrap(commands.saveAddNewFilesWhileScanning(enabled)).catch(reportError)}
          />
        </ReasonTooltip>
      </div>
      {/* "quickLookViewModel" (sic) is the Kotlin 1.3.0 wording, kept for parity; see the checklist's open points. */}
      <span className="body-small muted settings-date-text">
        A new document's date comes from its file name (such as 2024-03-15 or 15.03.2024), else from the file's creation date. You can change it in the
        quickLookViewModel.
      </span>
      <ActionButtons buttons={[{ label: "Detect missing dates", icon: EVENT, onClick: operations.detectDocumentDates }]} enabled={busyReason === null} busyReason={busyReason} />
    </AppCard>
  );
}
