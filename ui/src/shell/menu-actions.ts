import {commands, events} from "../ipc";
import {libraryCommands} from "../features/library";
import {operations} from "../session/operations";
import {commandHeld} from "../shared/platform";
import {goTo, navigation, setLogVisible, setPaletteOpen} from "./screens";
import {isTauri} from "./tauri";

/** Runs an entry of the native menu bar (ids from `src-tauri/src/menu.rs`). */
export function runMenuAction(id: string) {
  const library = libraryCommands.get();
  const actions: Record<string, () => void> = {
    scan: operations.scan,
    syncAll: operations.syncToAllDestinations,
    mergeAll: operations.mergeAllFolders,
    retryFailed: operations.retryFailedOperations,
    detectDates: operations.detectDocumentDates,
    checkArchive: operations.checkArchive,
    goTo: () => setPaletteOpen(true),
    openDashboard: () => goTo("dashboard"),
    openTasks: () => goTo("tasks"),
    openLibrary: () => goTo("library"),
    openBills: () => goTo("bills"),
    openContracts: () => goTo("contracts"),
    openTax: () => goTo("tax"),
    openCases: () => goTo("cases"),
    openEmergency: () => goTo("emergency"),
    openSettings: () => goTo("settings"),
    showLog: () => setLogVisible(!navigation.get().logVisible),
    toggleSidebar: () => library?.toggleSidebar(),
    focusSearch: () => library?.focusSearch(),
    quickLook: () => library?.toggleQuickLook(),
    selectAll: () => library?.selectAll(),
    clearSelection: () => library?.clearSelection(),
    newFolder: () => library?.newFolder(),
    renameFolder: () => library?.renameFolder(),
    deleteFolder: () => library?.deleteFolder(),
  };
  actions[id]?.();
}

/** Without Tauri (mock mode in a browser) there is no native menu; the same shortcuts are taken from the keyboard. */
const SHORTCUTS: Record<string, string> = {
  r: "scan", S: "syncAll", M: "mergeAll", k: "goTo", "1": "openDashboard", "2": "openTasks", "3": "openLibrary", "4": "openBills", "5": "openContracts", "6": "openTax", "7": "openCases", "8": "openEmergency", ",": "openSettings", l: "showLog", "\\": "toggleSidebar",
};

/** Follows the menu bar and keeps its check marks and enabled entries in step with the window. */
export async function connectMenu(): Promise<void> {
  if (!isTauri()) {
    window.addEventListener("keydown", (event) => {
      if (!commandHeld(event)) return;
      const id = SHORTCUTS[event.key];
      if (!id) return;
      event.preventDefault();
      runMenuAction(id);
    });
  }
  await events.menuEvent.listen(({ payload }) => runMenuAction(payload.id));
  const sync = () => void commands.setMenuContext(libraryCommands.get() !== null, navigation.get().logVisible);
  libraryCommands.subscribe(sync);
  navigation.subscribe(sync);
  sync();
}
