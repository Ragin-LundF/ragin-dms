import { commands, unwrap } from "../../../ipc";
import { report, reportError } from "../../../session/session";
import { dialogs } from "../../../shared/components";
import { createStore } from "../../../shared/store";
import { picker } from "../state/picker";

/** A folder name typed in place; nothing is written before it is saved (Kotlin `FolderNaming`). */
export type FolderNaming = { kind: "new"; parent: string; proposed: string } | { kind: "rename"; path: string };

/** What the user chose in the folder view (Kotlin `FolderViewStateModel`); which folders exist comes from disk. */
export type FolderViewState = { open: string; expanded: string[]; naming: FolderNaming | null; draft: string; query: string };

export const folderStore = createStore<FolderViewState>({ open: "", expanded: [], naming: null, draft: "", query: "" });

/** The folders on disk as the library screen last read them, for new names and the delete question. */
export const diskFoldersStore = createStore<string[]>([]);

export function parentOf(path: string): string {
  const slash = path.lastIndexOf("/");
  return slash < 0 ? "" : path.slice(0, slash);
}

export function nameOf(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1);
}

export function isAtOrBelow(path: string, folder: string): boolean {
  return path === folder || path.startsWith(`${folder}/`);
}

/** `path` moved below `to` if it lies at or below `from`, otherwise `path` itself. */
export function relocated(path: string, from: string, to: string): string {
  return isAtOrBelow(path, from) ? to + path.slice(from.length) : path;
}

function update(change: (state: FolderViewState) => Partial<FolderViewState>) {
  folderStore.set((state) => ({ ...state, ...change(state) }));
}

function moved(from: string, to: string) {
  update((state) => ({ open: relocated(state.open, from, to), expanded: state.expanded.map((path) => relocated(path, from, to)) }));
}

function childPath(parent: string, name: string): string {
  const cleaned = name.trim().replaceAll("/", "-");
  return parent ? `${parent}/${cleaned}` : cleaned;
}

/** The folder view's navigation and its folder actions (Kotlin `FolderBrowser` + `FolderEditor`). */
export const folderActions = {
  /** Opens `path` and unfolds the tree down to it; a name being typed is saved first. */
  open(path: string) {
    folderActions.saveName();
    const parents: string[] = [];
    for (let parent = parentOf(path); parent; parent = parentOf(parent)) parents.push(parent);
    update((state) => ({ open: path, query: "", expanded: [...new Set([...state.expanded, ...parents])] }));
  },
  toggle(path: string) {
    update((state) => ({ expanded: state.expanded.includes(path) ? state.expanded.filter((open) => open !== path) : [...state.expanded, path] }));
  },
  expand(path: string) {
    update((state) => ({ expanded: state.expanded.includes(path) ? state.expanded : [...state.expanded, path] }));
  },
  search(query: string) {
    update(() => ({ query }));
  },
  /** Opens `parent` with a new folder whose name is typed in place. */
  async newFolder(parent = folderStore.get().open) {
    folderActions.saveName();
    try {
      const proposed = await unwrap(commands.libraryProposedFolderName(parent, diskFoldersStore.get()));
      update((state) => ({ open: parent, naming: { kind: "new", parent, proposed }, draft: proposed, expanded: [...new Set([...state.expanded, parent])] }));
    } catch (error) {
      reportError(error);
    }
  },
  renameOpenFolder() {
    folderActions.rename(folderStore.get().open);
  },
  /** Opens the folder around `path` with `path`'s name ready to be typed over; the top level has no name. */
  rename(path: string) {
    if (!path) {
      report({ text: "The library's top level cannot be renamed", isError: true });
      return;
    }
    folderActions.saveName();
    update(() => ({ open: parentOf(path), naming: { kind: "rename", path }, draft: nameOf(path) }));
  },
  type(draft: string) {
    update(() => ({ draft }));
  },
  /** Saves the name being typed, if any; clicking away from the name does this. */
  saveName() {
    const { naming, draft } = folderStore.get();
    if (naming) folderActions.commitName(naming, draft);
  },
  /** Saves `text` as the name `naming` asked for; nothing happens when that name is no longer being typed. */
  commitName(naming: FolderNaming, text: string) {
    if (folderStore.get().naming !== naming) return;
    update(() => ({ naming: null }));
    if (naming.kind === "new") {
      unwrap(commands.libraryCreateFolder(naming.parent, text)).catch(reportError);
      return;
    }
    const target = childPath(parentOf(naming.path), text);
    unwrap(commands.libraryRenameFolder(naming.path, text))
      .then((done) => done && moved(naming.path, target))
      .catch(reportError);
  },
  cancelName() {
    update(() => ({ naming: null }));
  },
  /** Opens the picker to move `path`; the folder itself and its subfolders cannot be picked. */
  chooseTarget(path: string) {
    folderActions.saveName();
    picker.open(`Move "${nameOf(path)}" to…`, (folder) => folderActions.moveInto(path, folder), path);
  },
  moveInto(path: string, folder: string) {
    if (!isAtOrBelow(folder, path)) folderActions.expand(folder);
    unwrap(commands.libraryMoveFolderInto(path, folder))
      .then((done) => done && moved(path, childPath(folder, nameOf(path))))
      .catch(reportError);
  },
  deleteOpenFolder() {
    void folderActions.remove(folderStore.get().open);
  },
  /** Asks first, naming what really lies in `path`, then trashes it everywhere; the top level cannot be deleted. */
  async remove(path: string) {
    if (!path) {
      report({ text: "The library's top level cannot be deleted", isError: true });
      return;
    }
    folderActions.saveName();
    try {
      const request = await unwrap(commands.libraryFolderDeleteRequest(path, diskFoldersStore.get()));
      if (!request || !(await dialogs.confirm(request))) return;
      if (!(await unwrap(commands.libraryTrashFolder(path, request.affected.length)))) return;
      update((state) => ({
        open: isAtOrBelow(state.open, path) ? parentOf(path) : state.open,
        expanded: state.expanded.filter((folder) => !isAtOrBelow(folder, path)),
      }));
    } catch (error) {
      reportError(error);
    }
  },
};
