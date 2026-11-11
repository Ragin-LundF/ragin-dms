import FOLDER from "@material-design-icons/svg/round/folder.svg?raw";
import LIBRARY from "@material-design-icons/svg/round/library_books.svg?raw";
import { Button, Dialog, FoldToggle, Icon } from "../../../shared/components";
import { useStore } from "../../../shared/store";
import { picker, pickerStore, isExcluded } from "../state/picker";
import { useFolderTree } from "../state/queries";

/** The open "Move to…" picker: the top level and the folders as a tree, subfolders folded (Kotlin `FolderPickerDialog`). */
export function FolderPickerDialog({ foldersOnDisk }: { foldersOnDisk: string[] }) {
  const state = useStore(pickerStore);
  const rows = useFolderTree(foldersOnDisk, state?.expanded ?? [], state !== null);
  if (!state) return null;
  return (
    <Dialog
      title={state.title}
      width="var(--dialog-width)"
      onDismiss={picker.dismiss}
      buttons={
        <>
          <Button variant="text" onClick={picker.dismiss}>Cancel</Button>
          <Button variant="filled" disabled={state.selected === null} onClick={picker.confirm}>{state.confirmLabel}</Button>
        </>
      }
    >
      <div className="lib-picker">
        <PickerRow label="Library" icon={LIBRARY} depth={0} selected={state.selected === ""} excluded={false} expanded={null} onToggle={() => undefined} onSelect={() => picker.select("")} />
        {(rows.data ?? []).map((folder) => (
          <PickerRow
            key={folder.path}
            label={folder.name}
            icon={FOLDER}
            depth={folder.depth + 1}
            selected={state.selected === folder.path}
            excluded={isExcluded(state, folder.path)}
            expanded={folder.hasChildren ? folder.expanded : null}
            onToggle={() => picker.toggle(folder.path)}
            onSelect={() => picker.select(folder.path)}
          />
        ))}
      </div>
    </Dialog>
  );
}

function PickerRow({ label, icon, depth, selected, excluded, expanded, onToggle, onSelect }: {
  label: string;
  icon: string;
  depth: number;
  selected: boolean;
  excluded: boolean;
  expanded: boolean | null;
  onToggle: () => void;
  onSelect: () => void;
}) {
  const classes = ["lib-picker-row", selected ? "selected" : "", excluded ? "excluded" : ""].filter(Boolean).join(" ");
  return (
    <div role="option" aria-selected={selected} aria-disabled={excluded} className={classes} style={{ paddingLeft: 16 * depth }} onClick={excluded ? undefined : onSelect}>
      <FoldToggle expanded={expanded} onToggle={onToggle} />
      <Icon svg={icon} />
      <span className="body-medium ellipsis">{label}</span>
    </div>
  );
}
