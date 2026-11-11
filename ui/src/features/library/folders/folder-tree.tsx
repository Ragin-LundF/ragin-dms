import ADD from "@material-design-icons/svg/round/add.svg?raw";
import CREATE from "@material-design-icons/svg/round/create_new_folder.svg?raw";
import DELETE from "@material-design-icons/svg/round/delete.svg?raw";
import EDIT from "@material-design-icons/svg/round/edit.svg?raw";
import FOLDER from "@material-design-icons/svg/round/folder.svg?raw";
import FOLDER_OPEN from "@material-design-icons/svg/round/folder_open.svg?raw";
import LIBRARY from "@material-design-icons/svg/round/library_books.svg?raw";
import MORE from "@material-design-icons/svg/round/more_horiz.svg?raw";
import MOVE from "@material-design-icons/svg/round/drive_file_move.svg?raw";
import SIDEBAR from "@material-design-icons/svg/round/view_sidebar.svg?raw";
import { useEffect } from "react";
import type { LibraryFolderEntryDto } from "../../../ipc";
import { ActionMenu, FoldToggle, Icon, IconButton } from "../../../shared/components";
import { beginPress, useDropTarget } from "../interaction/drag";
import { panels } from "../state/panels";
import { folderActions } from "./folder-state";

/** A folded row unfolds after something hovered over it this long during a drag. */
const HOVER_OPEN_MS = 600;

/** The folder tree (Kotlin `FolderTreePane`): "Library" and its folders, + for a top-level folder, and the hide button. */
export function FolderTree({ open, tree, libraryCount, width, busyReason }: { open: string; tree: LibraryFolderEntryDto[]; libraryCount: number; width: number; busyReason: string | null }) {
  return (
    <div className="lib-tree" style={{ width }}>
      <div className="lib-tree-header">
        <span className="label-large grow muted" style={{ fontWeight: 600 }}>Folders</span>
        <IconButton icon={ADD} label="New folder at the top level" onClick={() => void folderActions.newFolder("")} />
        <TreeToggleButton hidden={false} />
      </div>
      <div className="lib-tree-rows">
        <FolderTreeRow path="" label="Library" icon={LIBRARY} depth={0} count={libraryCount} selected={open === ""} expanded={null} busyReason={null} />
        {tree.map((entry) => (
          <FolderTreeRow
            key={entry.path}
            path={entry.path}
            label={entry.name}
            icon={entry.path === open ? FOLDER_OPEN : FOLDER}
            depth={entry.depth - 1}
            count={entry.documentCount}
            selected={entry.path === open}
            expanded={entry.hasChildren ? entry.expanded : null}
            busyReason={busyReason}
          />
        ))}
      </div>
    </div>
  );
}

/** The panel button that hides the folder tree, or shows it again while it is `hidden`. */
export function TreeToggleButton({ hidden }: { hidden: boolean }) {
  return <IconButton icon={SIDEBAR} label={hidden ? "Show the folder tree" : "Hide the folder tree"} onClick={() => panels.toggleFolded("folderTree")} />;
}

/**
 * One row of the tree, indented by `depth` (Kotlin `FolderTreeRow`): a click opens the folder, drops move into it, a
 * folded row unfolds after a drag hovered over it a moment, and every folder but the top level can be dragged.
 */
function FolderTreeRow({ path, label, icon, depth, count, selected, expanded, busyReason }: {
  path: string;
  label: string;
  icon: string;
  depth: number;
  count: number;
  selected: boolean;
  expanded: boolean | null;
  busyReason: string | null;
}) {
  const drop = useDropTarget({ kind: "folder", path, place: "tree" });
  useEffect(() => {
    if (!drop.hovered || expanded !== false) return;
    const timer = window.setTimeout(() => folderActions.expand(path), HOVER_OPEN_MS);
    return () => window.clearTimeout(timer);
  }, [drop.hovered, expanded, path]);
  const classes = ["lib-tree-row", selected ? "selected" : "", drop.hovered ? "lib-drop-hover" : ""].filter(Boolean).join(" ");
  return (
    <div
      ref={drop.ref}
      role="treeitem"
      aria-selected={selected}
      aria-expanded={expanded ?? undefined}
      className={classes}
      style={{ paddingLeft: 4 + 14 * depth }}
      onPointerDown={path ? (event) => beginPress(event, { kind: "folder", path }) : undefined}
      onClick={() => folderActions.open(path)}
    >
      <FoldToggle expanded={expanded} onToggle={() => folderActions.toggle(path)} />
      <Icon svg={icon} />
      <span className="lib-tree-label body-medium">{label}</span>
      {count > 0 && <span className="label-small">{count}</span>}
      {path && (
        <span className="lib-hover-menu" onClick={(event) => event.stopPropagation()} onPointerDown={(event) => event.stopPropagation()}>
          <FolderMenu path={path} busyReason={busyReason} />
        </span>
      )}
    </div>
  );
}

/** The ⋯ menu of a folder, the same on a tree row and a tile (Kotlin `FolderMenu`); changes wait while busy. */
export function FolderMenu({ path, busyReason }: { path: string; busyReason: string | null }) {
  return (
    <ActionMenu
      icon={MORE}
      label={`Actions for ${path.slice(path.lastIndexOf("/") + 1)}`}
      actions={[
        { label: "Open", icon: FOLDER_OPEN, onClick: () => folderActions.open(path) },
        { label: "New subfolder", icon: CREATE, onClick: () => void folderActions.newFolder(path) },
        { label: "Rename", icon: EDIT, shortcut: "↵", disabledReason: busyReason, onClick: () => folderActions.rename(path) },
        { label: "Move to…", icon: MOVE, disabledReason: busyReason, onClick: () => folderActions.chooseTarget(path) },
        { label: "Delete…", icon: DELETE, shortcut: "⌫", disabledReason: busyReason, onClick: () => void folderActions.remove(path) },
      ]}
    />
  );
}
