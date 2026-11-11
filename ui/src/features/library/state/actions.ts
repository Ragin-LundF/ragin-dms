import {commands, unwrap} from "../../../ipc";
import {closeQuickLook} from "../../document";
import {reportError} from "../../../session/session";
import {dialogs} from "../../../shared/components";
import {chooseFiles} from "../../../shared/platform";
import type {LibrarySelection} from "../navigation";
import {picker} from "./picker";
import {shownStore} from "./queries";
import {chosen, selection, selectionStore} from "./selection";
import {filteredFolder, opening, paramsOf, viewStore} from "./view-state";

/** "1 document" or "N documents". */
export function documentsText(count: number): string {
  return count === 1 ? "1 document" : `${count} documents`;
}

/** A busy lane makes these actions do nothing, as in Kotlin; every other failure is reported. */
function unlessBusy(error: unknown) {
  if ((error as { code?: string }).code !== "busy") reportError(error);
}

function shownOrder(): string[] {
  return shownStore.get()?.order ?? [];
}

/** The ticked documents that are shown, in shown order. */
export function chosenIds(): string[] {
  return chosen(selectionStore.get(), shownOrder());
}

/** Opens `next` in the sidebar: no filters, no search text, no ticks, quick look closed. */
export function select(next: LibrarySelection) {
  viewStore.set((state) => opening(state, next));
  selection.clear();
  closeQuickLook();
}

/** The bulk bar's actions (Kotlin `BulkActions`); each clears the ticks once it started or had nothing to do. */
export const bulk = {
  assignCategory(categoryId: string | null, ids = chosenIds()) {
    clearWhenStarted(commands.libraryBulkAssignCategory(ids, categoryId));
  },
  addTag(tag: string, ids = chosenIds()) {
    clearWhenStarted(commands.libraryBulkAddTag(ids, tag));
  },
  async newTag() {
    const ids = chosenIds();
    const tag = await dialogs.askText({ title: `Tag ${documentsText(ids.length)}`, label: "Tag", initialValue: "", confirmLabel: "Tag" });
    if (tag !== null) bulk.addTag(tag, ids);
  },
  move() {
    const ids = chosenIds();
    picker.open(`Move ${documentsText(ids.length)} to…`, (target) => bulk.moveTo(target, ids));
  },
  moveTo(target: string, ids = chosenIds()) {
    clearWhenStarted(commands.libraryBulkMove(ids, target));
  },
  tagWithAi() {
    clearWhenStarted(commands.libraryBulkTagWithAi(chosenIds()));
  },
  async trash() {
    const ids = chosenIds();
    const rows = new Map((shownStore.get()?.groups ?? []).flatMap((group) => group.rows).map((row) => [row.id, row.path]));
    const confirmed = await dialogs.confirm({
      title: `Move ${documentsText(ids.length)} to the trash?`,
      message: "They are moved to the system trash from every library folder; you can restore them from there.",
      affected: ids.map((id) => rows.get(id) ?? id),
      confirmLabel: "Move to trash",
    });
    if (confirmed) clearWhenStarted(commands.libraryBulkTrash(ids));
  },
};

function clearWhenStarted(started: ReturnType<typeof commands.libraryBulkMove>) {
  unwrap(started)
    .then((clear) => clear && selection.clear())
    .catch(unlessBusy);
}

async function askName(title: string, initialValue: string): Promise<string | null> {
  const name = await dialogs.askText({ title, label: "Name", initialValue, confirmLabel: "Save" });
  const trimmed = name?.trim();
  return trimmed ? trimmed : null;
}

/** Saved views of the sidebar (Kotlin `SmartFolders`). */
export const savedViews = {
  /** The view is captured when "Save view" is clicked, before the name is asked. */
  async saveCurrent() {
    const params = paramsOf(viewStore.get());
    const name = await askName("Save view", "");
    if (name) unwrap(commands.librarySaveView(params, name)).catch(reportError);
  },
  replaceWithCurrent(id: string) {
    unwrap(commands.libraryReplaceSavedView(id, paramsOf(viewStore.get()))).catch(reportError);
  },
  async rename(id: string, current: string) {
    const name = await askName("Rename saved view", current);
    if (name) unwrap(commands.libraryRenameSavedView(id, name)).catch(reportError);
  },
  async remove(id: string, name: string) {
    const confirmed = await dialogs.confirm({
      title: `Delete saved view "${name}"`,
      message: "Only the saved search is deleted; no document changes.",
      affected: [],
      confirmLabel: "Delete",
    });
    if (confirmed) unwrap(commands.libraryDeleteSavedView(id)).catch(reportError);
  },
  addStandard() {
    unwrap(commands.libraryAddStandardSavedViews()).catch(reportError);
  },
};

/** Creating, renaming and deleting categories and tags from the facet popovers (Kotlin `TaxonomyController`). */
export const taxonomy = {
  async newCategory(parentId: string | null) {
    const name = await dialogs.askText({ title: "New category", label: "Name", initialValue: "", confirmLabel: "Create" });
    if (name !== null) unwrap(commands.libraryAddCategory(name, parentId)).catch(reportError);
  },
  async renameCategory(categoryId: string, current: string) {
    const name = await dialogs.askText({ title: "Rename category", label: "Name", initialValue: current, confirmLabel: "Rename" });
    if (name !== null) unwrap(commands.libraryRenameCategory(categoryId, name)).catch(reportError);
  },
  async removeCategory(categoryId: string, name: string) {
    const confirmed = await dialogs.confirm({
      title: `Delete category "${name}"`,
      message: "The category leaves the catalog. No document uses it and no file changes.",
      affected: [],
      confirmLabel: "Delete category",
    });
    if (!confirmed) return;
    unwrap(commands.libraryRemoveCategory(categoryId))
      .then((done) => done && select({ kind: "folder", path: "" }))
      .catch(reportError);
  },
  async renameTag(tag: string) {
    const name = await dialogs.askText({ title: "Rename tag", label: "Tag", initialValue: tag, confirmLabel: "Rename" });
    if (name === null) return;
    unwrap(commands.libraryRenameTag(tag, name))
      .then((done) => done && select({ kind: "tag", tag: name.trim() }))
      .catch(reportError);
  },
  async removeTag(tag: string) {
    try {
      const request = await unwrap(commands.libraryRemoveTagRequest(tag));
      if (!(await dialogs.confirm(request))) return;
      if (await unwrap(commands.libraryRemoveTag(tag))) select({ kind: "folder", path: "" });
    } catch (error) {
      reportError(error);
    }
  },
};

/** Copies files into `target` of every library folder (Kotlin `FileAdder`). */
export function addFiles(paths: string[], target = filteredFolder(viewStore.get())) {
  unwrap(commands.libraryAddFiles(paths, target)).catch(unlessBusy);
}

/** Asks for files and adds them to `target`; cancelling adds nothing. */
export async function chooseAndAdd(target: string) {
  const files = await chooseFiles("Add files to the library");
  if (files.length > 0) addFiles(files, target);
}

