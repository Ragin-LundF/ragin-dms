import {closeQuickLook, openQuickLook, quickLook, quickLookEditing, stepQuickLook} from "../../document";
import {shownStore} from "../state/queries";
import {selection, selectionStore} from "../state/selection";
import {searchFocus, viewStore} from "../state/view-state";
import {folderActions} from "../folders/folder-state";

/** Marks the library's search field: Esc typed there still reaches the library, as in Kotlin. */
export const SEARCH_FIELD_ATTRIBUTE = "data-library-search";

/**
 * The library's keys outside the text fields (Kotlin `LibraryKeys`): quick look keys first, then the folder view's
 * keys while "Folders" is selected, then the document keys. While quick look edits a value every key belongs to that
 * field. Returns whether the key was the library's.
 */
export function handleLibraryKey(event: KeyboardEvent | React.KeyboardEvent): boolean {
  if (quickLookEditing.get() !== null) return false;
  const target = event.target as HTMLElement | null;
  const typing = target !== null && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable);
  if (typing && !(event.key === "Escape" && target.hasAttribute(SEARCH_FIELD_ATTRIBUTE))) return false;
  const command = event.metaKey || event.ctrlKey;
  const open = quickLook.get().documentId !== null;
  if (event.key === " " && !command) {
    toggleQuickLook();
    return true;
  }
  if ((event.key === "ArrowDown" || event.key === "ArrowUp") && open) {
    stepQuickLook(event.key === "ArrowDown" ? 1 : -1);
    return true;
  }
  if (viewStore.get().view.kind === "folders") {
    if (event.key === "Enter") {
      folderActions.renameOpenFolder();
      return true;
    }
    if (event.key === "Backspace" || event.key === "Delete") {
      folderActions.deleteOpenFolder();
      return true;
    }
  }
  if (event.key === "/" && !command) {
    focusSearch();
    return true;
  }
  if (event.key.toLowerCase() === "a" && command) {
    selection.selectAll(shownStore.get()?.order ?? []);
    return true;
  }
  if (event.key === "Escape") return escape();
  return false;
}

/** Space: closes quick look while it is open, or opens it on the first ticked document, else the first shown. */
export function toggleQuickLook() {
  if (quickLook.get().documentId !== null) {
    closeQuickLook();
    return;
  }
  const order = shownStore.get()?.order ?? [];
  const ticked = selectionStore.get().selected;
  // The first ticked one in the order it was ticked, as Kotlin's selection set keeps it.
  const first = [...ticked].find((id) => order.includes(id)) ?? order[0];
  if (first !== undefined) openQuickLook(first, order);
}

export function focusSearch() {
  searchFocus.set((count) => count + 1);
}

/** Esc closes quick look first, then clears the ticks; with neither the key is not the library's. */
function escape(): boolean {
  if (quickLook.get().documentId !== null) {
    closeQuickLook();
    return true;
  }
  const order = shownStore.get()?.order ?? [];
  if (![...selectionStore.get().selected].some((id) => order.includes(id))) return false;
  selection.clear();
  return true;
}
