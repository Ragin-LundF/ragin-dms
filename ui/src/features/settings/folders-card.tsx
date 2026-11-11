import FOLDER_SPECIAL from "@material-design-icons/svg/round/folder_special.svg?raw";
import FOLDER from "@material-design-icons/svg/round/folder.svg?raw";
import STAR from "@material-design-icons/svg/round/star.svg?raw";
import DELETE from "@material-design-icons/svg/round/delete.svg?raw";
import CREATE_NEW_FOLDER from "@material-design-icons/svg/round/create_new_folder.svg?raw";
import STORAGE from "@material-design-icons/svg/round/storage.svg?raw";
import { commands, unwrap, type FolderRowDto_Serialize, type LibraryCommandDto } from "../../ipc";
import { reportError } from "../../session/session";
import { ActionButtons, AppCard, CardTitle, IconBadge, LabelChip, dialogs } from "../../shared/components";
import { chooseFolder } from "../../shared/platform";

/** Runs a folder change as "Updating the library folders"; the library reopens when it worked. */
function configure(command: LibraryCommandDto) {
  void unwrap(commands.configureLibrary(command)).catch(reportError);
}

async function choosePrimaryFolder() {
  const location = await chooseFolder("Choose the primary library folder");
  if (location) configure({ kind: "createPrimary", location });
}

async function addSyncFolder() {
  const location = await chooseFolder("Choose a sync folder");
  if (location) configure({ kind: "addSyncFolder", location });
}

async function removeSyncFolder(row: FolderRowDto_Serialize) {
  const confirmed = await dialogs.confirm({
    title: "Remove sync folder",
    message: "The tool stops writing to this folder on this device. Its files stay where they are.",
    affected: [row.location],
    confirmLabel: "Remove",
  });
  if (confirmed) configure({ kind: "removeSyncFolder", folderId: row.id });
}

/** The first thing a new user sees: what the primary folder is, and choosing it. */
export function WelcomeCard({ busyReason }: { busyReason: string | null }) {
  return (
    <AppCard className="settings-card" padding={32}>
      <div className="column gap-l">
        <span className="headline-large">Welcome</span>
        <span className="body-large muted">
          Choose the folder that holds your documents. It becomes the primary folder: every change is written there first, then to every sync folder. Your
          files stay readable files on disk, with their original names.
        </span>
        <ActionButtons
          buttons={[{ label: "Choose primary folder…", icon: FOLDER_SPECIAL, style: "filled", onClick: () => void choosePrimaryFolder() }]}
          enabled={busyReason === null}
          busyReason={busyReason}
        />
      </div>
    </AppCard>
  );
}

/** The primary folder and the sync folders, with the actions that change them. */
export function FoldersCard({ rows, busyReason }: { rows: FolderRowDto_Serialize[]; busyReason: string | null }) {
  return (
    <AppCard className="settings-card">
      <CardTitle title="Library folders" icon={STORAGE} tint="var(--secondary)" subtitle="Every change is written to the primary first, then to each sync folder" />
      <div className="column gap-s">
        {rows.map((row) => <FolderRow key={row.id} row={row} busyReason={busyReason} />)}
      </div>
      <span className="body-small muted settings-hint">A sync folder can be any folder, such as one a cloud service keeps in sync.</span>
      <ActionButtons
        buttons={[{ label: "Add sync folder…", icon: CREATE_NEW_FOLDER, style: "tonal", onClick: () => void addSyncFolder() }]}
        enabled={busyReason === null}
        busyReason={busyReason}
      />
    </AppCard>
  );
}

/** One library folder with its role and the actions it allows. */
function FolderRow({ row, busyReason }: { row: FolderRowDto_Serialize; busyReason: string | null }) {
  const tint = row.primary ? "var(--primary)" : "var(--secondary)";
  return (
    <div className="settings-folder-row">
      <IconBadge icon={row.primary ? FOLDER_SPECIAL : FOLDER} tint={tint} />
      <div className="column grow">
        <div className="row gap-s">
          <span className="title-small ellipsis">{row.displayName}</span>
          <LabelChip text={row.primary ? "Primary" : "Sync folder"} color={tint} />
        </div>
        <span className="body-small muted ellipsis">{row.location}</span>
        {row.note && <span className="body-small error-text">{row.note}</span>}
      </div>
      {!row.primary && (
        <ActionButtons
          buttons={[
            { label: "Make primary", icon: STAR, style: "text", onClick: () => configure({ kind: "choosePrimary", folderId: row.id }) },
            { label: "Remove", icon: DELETE, style: "text", onClick: () => void removeSyncFolder(row) },
          ]}
          enabled={busyReason === null}
          busyReason={busyReason}
        />
      )}
    </div>
  );
}
