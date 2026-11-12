import type {HealthFolderDto} from "../../../ipc";
import {SectionHeader} from "../../../shared/components";
import {TONE_COLOR} from "./tone";

/** One row per library folder: name, location, what the last check read there, and its state. */
export function HealthFolders({ folders }: { folders: HealthFolderDto[] }) {
  return (
    <div className="column gap-s">
      <SectionHeader title="Library folders" />
      {folders.map((folder) => (
        <div key={folder.location} className="health-folder-row">
          <div className="column grow">
            <span className="title-small ellipsis">{folder.primary ? `${folder.name} · primary` : folder.name}</span>
            <span className="body-small muted ellipsis">{folder.location}</span>
          </div>
          <span className="body-small muted nowrap">{folder.summary}</span>
          <span className="health-pill nowrap" style={{ background: `color-mix(in srgb, ${TONE_COLOR[folder.tone]} 18%, transparent)` }}>
            {folder.pill}
          </span>
        </div>
      ))}
    </div>
  );
}
