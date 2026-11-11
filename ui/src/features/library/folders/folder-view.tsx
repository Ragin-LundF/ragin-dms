import ADD from "@material-design-icons/svg/round/add.svg?raw";
import CREATE from "@material-design-icons/svg/round/create_new_folder.svg?raw";
import DELETE from "@material-design-icons/svg/round/delete.svg?raw";
import EDIT from "@material-design-icons/svg/round/edit.svg?raw";
import FOLDER from "@material-design-icons/svg/round/folder.svg?raw";
import { useEffect, useRef } from "react";
import type { LibraryCrumbDto, LibraryFolderTileDto, LibraryFolderViewDto_Serialize } from "../../../ipc";
import { openQuickLook } from "../../document";
import { ActionButtons, DropOverlay, IconBadge, ResizeHandle, type ActionButton } from "../../../shared/components";
import { useStore } from "../../../shared/store";
import { beginPress, useDropTarget } from "../interaction/drag";
import { useFileDrop } from "../interaction/file-drop";
import { DocumentGrid, type DocumentCallbacks } from "../documents/document-grid";
import { TokenSearchField } from "../filter/token-search-field";
import { addFiles, chooseAndAdd } from "../state/actions";
import { panelOf, panelStore, panels } from "../state/panels";
import { useFolderView } from "../state/queries";
import { folderActions, folderStore, type FolderNaming } from "./folder-state";
import { FolderMenu, FolderTree, TreeToggleButton } from "./folder-tree";

/**
 * The folder view, a file manager of the library's folders (Kotlin `FolderView`): the tree on the left (resizable,
 * hideable), and on the right a search of the open folder, its path and actions, its subfolders and its documents,
 * drawn as the library's cards. A click elsewhere takes the focus from a name being typed, which saves it.
 */
export function FolderView({ foldersOnDisk, callbacks, busyReason }: { foldersOnDisk: string[]; callbacks: DocumentCallbacks; busyReason: string | null }) {
  const folders = useStore(folderStore);
  const tree = panelOf(useStore(panelStore), "folderTree");
  const view = useFolderView({ open: folders.open, expanded: folders.expanded, query: folders.query, foldersOnDisk }).data;
  if (!view) return null;
  return (
    <div className="lib-folders">
      {!tree.folded && (
        <>
          <FolderTree open={folders.open} tree={view.tree} libraryCount={view.libraryCount} width={tree.width} busyReason={busyReason} />
          <ResizeHandle onResize={(delta) => panels.resize("folderTree", delta)} onResizeEnd={() => panels.endResize("folderTree")} onDoubleClick={() => panels.resetWidth("folderTree")} />
        </>
      )}
      <div className="lib-folder-main">
        <TokenSearchField query={folders.query} tokens={[]} suggestions={[]} placeholder={view.searchPlaceholder} onQuery={folderActions.search} onRemove={() => undefined} onTake={() => undefined} />
        <FolderHeader open={folders.open} crumbs={view.crumbs} treeHidden={tree.folded} busyReason={busyReason} />
        <FolderTiles tiles={view.tiles} naming={folders.naming} busyReason={busyReason} />
        <FolderDocuments open={folders.open} view={view} callbacks={callbacks} />
      </div>
    </div>
  );
}

/** The path of the open folder, each segment opening its folder and taking drops, and the open folder's actions. */
function FolderHeader({ open, crumbs, treeHidden, busyReason }: { open: string; crumbs: LibraryCrumbDto[]; treeHidden: boolean; busyReason: string | null }) {
  const newFolder: ActionButton = { label: "New folder", icon: CREATE, onClick: () => void folderActions.newFolder() };
  const addHere: ActionButton = { label: "Add files here", icon: ADD, style: "filled", onClick: () => void chooseAndAdd(open) };
  const buttons = open
    ? [newFolder, { label: "Rename", icon: EDIT, onClick: folderActions.renameOpenFolder }, { label: "Delete", icon: DELETE, onClick: folderActions.deleteOpenFolder }, addHere]
    : [newFolder, addHere];
  return (
    <div className="lib-folder-header">
      {treeHidden && <TreeToggleButton hidden />}
      <div className="lib-crumbs">
        {crumbs.map((crumb, index) => (
          <span key={crumb.path} className="row" style={{ minWidth: 0 }}>
            {index > 0 && <span className="lib-crumb-separator title-medium">/</span>}
            <Crumb crumb={crumb} last={index === crumbs.length - 1} />
          </span>
        ))}
      </div>
      <ActionButtons buttons={buttons} enabled={busyReason === null} busyReason={busyReason} />
    </div>
  );
}

function Crumb({ crumb, last }: { crumb: LibraryCrumbDto; last: boolean }) {
  const drop = useDropTarget({ kind: "folder", path: crumb.path, place: "breadcrumb" });
  const classes = ["lib-crumb", "title-medium", last ? "last" : "", drop.hovered ? "lib-drop-hover" : ""].filter(Boolean).join(" ");
  return (
    <span ref={drop.ref} role="link" className={classes} onClick={() => folderActions.open(crumb.path)}>
      {crumb.name}
    </span>
  );
}

/** The "Folders" section: a tile per subfolder and the tile of a new folder while its name is typed. */
function FolderTiles({ tiles, naming, busyReason }: { tiles: LibraryFolderTileDto[]; naming: FolderNaming | null; busyReason: string | null }) {
  const adding = naming?.kind === "new" ? naming : null;
  if (tiles.length === 0 && !adding) return null;
  return (
    <div className="column gap-s" style={{ flex: "none" }}>
      <SectionTitle title="Folders" count={tiles.length} />
      <div className="lib-folder-tiles">
        {adding && (
          <div className="lib-folder-tile">
            <IconBadge icon={FOLDER} tint="var(--primary)" size={32} />
            <div className="column grow"><NameField naming={adding} initial={adding.proposed} /></div>
          </div>
        )}
        {tiles.map((tile) => (
          <FolderTile key={tile.path} tile={tile} renaming={naming?.kind === "rename" && naming.path === tile.path ? naming : null} busyReason={busyReason} />
        ))}
      </div>
    </div>
  );
}

function FolderTile({ tile, renaming, busyReason }: { tile: LibraryFolderTileDto; renaming: FolderNaming | null; busyReason: string | null }) {
  const drop = useDropTarget({ kind: "folder", path: tile.path, place: "tile" });
  return (
    <div
      ref={drop.ref}
      className={drop.hovered ? "lib-folder-tile lib-drop-hover" : "lib-folder-tile"}
      onPointerDown={renaming ? undefined : (event) => beginPress(event, { kind: "folder", path: tile.path })}
      onClick={() => folderActions.open(tile.path)}
    >
      <IconBadge icon={FOLDER} tint="var(--primary)" size={32} />
      <div className="column grow">
        {renaming ? <NameField naming={renaming} initial={tile.name} /> : <span className="body-medium ellipsis" style={{ fontWeight: 500 }}>{tile.name}</span>}
        <span className="body-small muted ellipsis">{tile.caption}</span>
      </div>
      <span className="lib-hover-menu" onClick={(event) => event.stopPropagation()} onPointerDown={(event) => event.stopPropagation()}>
        <FolderMenu path={tile.path} busyReason={busyReason} />
      </span>
    </div>
  );
}

/**
 * A folder name typed in place, focused and selected when it appears (Kotlin `FolderNameField`). Enter or losing the
 * focus saves it and Esc cancels, whichever comes first.
 */
function NameField({ naming, initial }: { naming: FolderNaming; initial: string }) {
  const input = useRef<HTMLInputElement>(null);
  const done = useRef(false);
  useEffect(() => {
    input.current?.focus();
    input.current?.select();
  }, []);
  const finish = (action: () => void) => {
    if (done.current) return;
    done.current = true;
    action();
  };
  return (
    <input
      ref={input}
      className="lib-name-field body-medium"
      defaultValue={initial}
      aria-label="Folder name"
      onClick={(event) => event.stopPropagation()}
      onChange={(event) => folderActions.type(event.target.value)}
      onBlur={(event) => finish(() => folderActions.commitName(naming, event.target.value))}
      onKeyDown={(event) => {
        if (event.key === "Enter") finish(() => folderActions.commitName(naming, event.currentTarget.value));
        if (event.key === "Escape") finish(folderActions.cancelName);
        if (event.key === "Enter" || event.key === "Escape") event.stopPropagation();
      }}
    />
  );
}

/** The "Documents" section: the documents directly in the open folder, or a drop zone saying there are none. */
function FolderDocuments({ open, view, callbacks }: { open: string; view: LibraryFolderViewDto_Serialize; callbacks: DocumentCallbacks }) {
  const zone = useRef<HTMLDivElement>(null);
  const dropping = useFileDrop(zone, (paths) => addFiles(paths, open));
  const order = view.documents.map((row) => row.id);
  return (
    <div className="lib-folder-documents">
      <SectionTitle title="Documents" count={view.documents.length} hint="Drag documents onto a folder, the tree or the path above to move them" />
      <div ref={zone} className="lib-folder-drop">
        {view.documents.length === 0 ? (
          <div className="lib-empty-drop body-medium">{view.emptyText}</div>
        ) : (
          <DocumentGrid groups={[{ title: "", rows: view.documents }]} headers={false} {...callbacks} onClick={(id) => openQuickLook(id, order)} />
        )}
        {dropping && <DropOverlay label={`Add to ${open || "the top level"}`} />}
      </div>
    </div>
  );
}

function SectionTitle({ title, count, hint }: { title: string; count: number; hint?: string }) {
  return (
    <div className="lib-section-title">
      <span className="label-large muted" style={{ fontWeight: 600 }}>{title}</span>
      <span className="label-medium" style={{ color: "var(--outline)" }}>{count}</span>
      {hint && <span className="label-medium ellipsis grow" style={{ color: "var(--outline)", textAlign: "end" }}>{hint}</span>}
    </div>
  );
}
