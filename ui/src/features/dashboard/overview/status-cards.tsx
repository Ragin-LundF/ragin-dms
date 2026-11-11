import STORAGE from "@material-design-icons/svg/round/storage.svg?raw";
import FOLDER from "@material-design-icons/svg/round/folder.svg?raw";
import FOLDER_SPECIAL from "@material-design-icons/svg/round/folder_special.svg?raw";
import TIMELINE from "@material-design-icons/svg/round/timeline.svg?raw";
import type {ActivityDto, FolderHealth, FolderStatusDto} from "../../../ipc";
import {AppCard, CardTitle, IconBadge} from "../../../shared/components";

const HEALTH_COLOR: Record<FolderHealth, string> = {
  reachable: "var(--status-good)",
  offline: "var(--error)",
  foreign: "var(--status-warning)",
  notScanned: "var(--outline)",
};

/** The library folders with their role and whether they can be reached (Kotlin `FolderPanel`). */
export function FolderPanel({ folders }: { folders: FolderStatusDto[] }) {
  return (
    <AppCard className="status-card">
      <CardTitle title="Library folders" icon={STORAGE} tint="var(--secondary)" />
      <div className="column gap-m">
        {folders.map((folder) => (
          <div key={folder.location} className="row gap-m">
            <IconBadge icon={folder.primary ? FOLDER_SPECIAL : FOLDER} tint={folder.primary ? "var(--primary)" : "var(--secondary)"} size={32} />
            <div className="column grow">
              <span className="title-small ellipsis">{folder.title}</span>
              <span className="body-small muted ellipsis">{folder.location}</span>
            </div>
            <span className="status-dot" style={{ background: HEALTH_COLOR[folder.health] }} />
            <span className="label-small muted nowrap">{folder.status}</span>
          </div>
        ))}
      </div>
    </AppCard>
  );
}

/** What the tool did recently on this device, newest first (Kotlin `ActivityPanel`). */
export function ActivityPanel({ entries }: { entries: ActivityDto[] }) {
  return (
    <AppCard className="status-card">
      <CardTitle title="Recent activity" icon={TIMELINE} tint="var(--tertiary)" />
      {entries.length === 0 && <span className="body-medium muted">Nothing yet</span>}
      <div className="column gap-s scroll">
        {entries.map((entry, index) => (
          <div key={`${entry.time}-${index}`} className="activity-row">
            <span className="activity-dot" />
            <div className="column grow">
              <span className="body-medium ellipsis">{entry.line}</span>
              <span className="label-small muted">{entry.time}</span>
            </div>
          </div>
        ))}
      </div>
    </AppCard>
  );
}
