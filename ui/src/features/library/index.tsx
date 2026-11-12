import {chooseAndAdd} from "./state/actions";
import {filteredFolder, viewStore} from "./state/view-state";
import "./library.css";

export { LibraryScreen } from "./library-screen";
export { libraryCommands, libraryTitle, type LibraryCommands } from "./library-stores";

/** "Add files" of the dashboard: asks for files and adds them at the top level of the library. */
export async function addFilesToTopLevel(): Promise<void> {
  await chooseAndAdd("");
}

/** "Add files" of the library: asks for files and adds them to the folder the library filters by, else the top level. */
export async function addFilesToOpenFolder(): Promise<void> {
  await chooseAndAdd(filteredFolder(viewStore.get()));
}

export { ExportDialog } from "./export/export-dialog";
export { openTaxAdviserExport } from "./export/export-state";
