import DRIVE_FILE_MOVE from "@material-design-icons/svg/round/drive_file_move.svg?raw";
import CREATE_NEW_FOLDER from "@material-design-icons/svg/round/create_new_folder.svg?raw";
import FOLDER from "@material-design-icons/svg/round/folder.svg?raw";
import {Button, Chip, Dialog, Icon, TextField} from "../../shared/components";

/** The open "Move to…" picker for one document: the folder typed or picked, and the file name to keep or change. */
export type FolderPick = { documentId: string; documentName: string; folder: string; fileName: string };

/** The typed folder without surrounding blanks and slashes. */
export function typedFolder(folder: string): string {
  return folder.trim().replace(/^\/+|\/+$/g, "");
}

/**
 * "Move to…": a folder of the library, searched by typing, or a new path, which is created; recently used folders
 * come first. The file name can be changed on the way.
 */
export function FolderPickerDialog({ pick, folders, recent, onChange, onConfirm, onDismiss }: {
  pick: FolderPick;
  folders: string[];
  recent: string[];
  onChange: (folder: string, fileName: string) => void;
  onConfirm: () => void;
  onDismiss: () => void;
}) {
  const typed = typedFolder(pick.folder);
  const lower = typed.toLowerCase();
  const matches = folders.filter((folder) => folder.toLowerCase().includes(lower));
  const isNew = typed !== "" && !folders.some((folder) => folder.toLowerCase() === lower);
  const ready = typed !== "" && pick.fileName.trim() !== "";
  return (
    <Dialog
      width={560}
      onDismiss={onDismiss}
      title={
        <span className="inbox-dialog-head">
          <Icon svg={DRIVE_FILE_MOVE} />
          <span>Move {pick.documentName}</span>
        </span>
      }
      buttons={
        <>
          <Button variant="text" onClick={onDismiss}>Cancel</Button>
          <Button variant="filled" disabled={!ready} onClick={onConfirm}>Move</Button>
        </>
      }
    >
      <div className="column gap-m">
        {recent.length > 0 && (
          <>
            <span className="label-large">Recently used</span>
            <div className="row wrap gap-s">
              {recent.map((folder) => <Chip key={folder} label={<span className="nowrap">{folder}</span>} onClick={() => onChange(folder, pick.fileName)} />)}
            </div>
          </>
        )}
        <TextField
          label="Folder: search, pick, or type a new path"
          value={pick.folder}
          onChange={(folder) => onChange(folder, pick.fileName)}
          leading={<span className="field-icon"><Icon svg={isNew ? CREATE_NEW_FOLDER : FOLDER} /></span>}
          support={isNew ? `"${typed}" will be created in every library folder` : undefined}
          labelBackground="var(--surface-high)"
          autoFocus
        />
        <div className="inbox-folder-list scroll">
          {matches.map((folder) => (
            <button key={folder} type="button" className="inbox-folder-option body-large ellipsis" onClick={() => onChange(folder, pick.fileName)}>
              {folder}
            </button>
          ))}
        </div>
        <TextField label="File name" value={pick.fileName} onChange={(fileName) => onChange(pick.folder, fileName)} labelBackground="var(--surface-high)" />
      </div>
    </Dialog>
  );
}
