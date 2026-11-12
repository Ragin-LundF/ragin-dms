import {useState} from "react";
import {commands, type HealthSnapshotDto, unwrap} from "../../../ipc";
import {reportError} from "../../../session/session";
import {Button, Dialog, dialogs, ReasonTooltip, SectionHeader} from "../../../shared/components";
import {absoluteDate, timeOfDay} from "../../../shared/format/dates";

type Comparison = { snapshot: HealthSnapshotDto; lines: string[] };

function takenAt(snapshot: HealthSnapshotDto): string {
  return `${absoluteDate(snapshot.takenAt)} ${timeOfDay(snapshot.takenAt)}`;
}

/** The catalog snapshots of this device, newest first: Compare shows what restoring changes, Restore… asks first. */
export function HealthSnapshots({ snapshots, busyReason }: { snapshots: HealthSnapshotDto[]; busyReason: string | null }) {
  const [comparison, setComparison] = useState<Comparison | null>(null);
  const compare = (snapshot: HealthSnapshotDto) =>
    unwrap(commands.healthCompareSnapshot(snapshot.id)).then((diff) => setComparison({ snapshot, lines: diff.lines }), reportError);
  const restore = async (snapshot: HealthSnapshotDto) => {
    const diff = await unwrap(commands.healthCompareSnapshot(snapshot.id)).catch(reportError);
    if (!diff) return;
    const confirmed = await dialogs.confirm({
      title: `Restore the catalog of ${takenAt(snapshot)}`,
      message: "The catalog goes back to this snapshot as a new revision, in every folder and on every device. Files are not touched; a scan afterwards shows where they differ.",
      affected: diff.lines,
      confirmLabel: "Restore",
    });
    if (confirmed) await unwrap(commands.healthRestoreSnapshot(snapshot.id)).catch(reportError);
  };
  return (
    <div className="column gap-s">
      <SectionHeader title="Catalog snapshots" />
      {snapshots.length === 0 && <span className="body-medium muted">A snapshot is taken after every change of the catalog.</span>}
      {snapshots.map((snapshot) => (
        <div key={snapshot.id} className="health-snapshot">
          <div className="column grow">
            <span className="title-small">{takenAt(snapshot)}</span>
            <span className="body-small muted">{snapshot.current ? `${snapshot.label} · the current catalog` : snapshot.label}</span>
          </div>
          <Button variant="text" disabled={snapshot.current} onClick={() => void compare(snapshot)}>Compare</Button>
          <ReasonTooltip reason={snapshot.current ? "This is the current catalog" : busyReason}>
            <Button disabled={snapshot.current || busyReason !== null} onClick={() => void restore(snapshot)}>Restore…</Button>
          </ReasonTooltip>
        </div>
      ))}
      {comparison && (
        <Dialog
          title={`Compared with ${takenAt(comparison.snapshot)}`}
          onDismiss={() => setComparison(null)}
          buttons={<Button variant="filled" onClick={() => setComparison(null)}>Close</Button>}
        >
          <ul className="column gap-xs body-medium">
            {comparison.lines.map((line) => <li key={line}>{line}</li>)}
          </ul>
        </Dialog>
      )}
    </div>
  );
}
