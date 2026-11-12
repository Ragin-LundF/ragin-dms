import type {LibraryFilterDto_Serialize} from "../../ipc";
import {goTo} from "../../shell/screens";

/**
 * What the library opens (Kotlin `LibrarySelection`). The sidebar's views are `folder` at the root ("All
 * documents"), `inbox`, `needsReview`, `recent`, `folders` and `smartFolder`. A `category`, `uncategorized`, `tag`
 * or a folder below the root, as the dashboard and the palette open them, become filters of "All documents".
 */
export type LibrarySelection =
  | { kind: "folder"; path: string }
  | { kind: "category"; categoryId: string }
  | { kind: "uncategorized" }
  | { kind: "tag"; tag: string }
  | { kind: "inbox" }
  | { kind: "needsReview" }
  | { kind: "recent" }
  | { kind: "folders" }
  | { kind: "smartFolder"; id: string };

/**
 * A request to the library screen: show a selection (with a search and a year, `yearAndEarlier` for that year and
 * every earlier one, as the year chart's "≤2017" bar, and further `filters`), or reveal one document.
 */
export type LibraryRequest =
  | { kind: "show"; selection: LibrarySelection; query: string; year?: number | null; yearAndEarlier?: boolean; filters?: LibraryFilterDto_Serialize[] }
  | { kind: "reveal"; documentId: string };

/** Shows the library at `request` (Kotlin `library.show(...)` / `library.reveal(...)` + switching the screen). */
export function openLibrary(request: LibraryRequest) {
  goTo("library", request);
}

/** Opens a document in the library: selects it and shows it in quick look. */
export function openDocument(documentId: string) {
  openLibrary({ kind: "reveal", documentId });
}
