import {createStore} from "../../shared/store";

/** The library's title for the window toolbar (Kotlin `LibraryUiStateModel.title`); set by the library screen. */
export const libraryTitle = createStore("Library");

/**
 * What the "Documents" menu does while the library screen is shown (Kotlin `LibraryCommands`); the library screen
 * sets it on mount and clears it on unmount, so the menu entries are disabled without it.
 */
export type LibraryCommands = {
  toggleSidebar: () => void;
  focusSearch: () => void;
  toggleQuickLook: () => void;
  selectAll: () => void;
  clearSelection: () => void;
  newFolder: () => void;
  renameFolder: () => void;
  deleteFolder: () => void;
};
export const libraryCommands = createStore<LibraryCommands | null>(null);
