import {emit} from "@tauri-apps/api/event";
import type {SessionState_Serialize} from "../../ipc";
import type {Handler} from "./index";

/** The mocked session: an open library with a primary and a sync folder. */
export const mockSession: { state: SessionState_Serialize } = {
  state: {
    status: { kind: "open", foreignFolders: [], unreachableFolders: [] },
    folders: [
      { id: "f-primary", location: "/Synthetic/Documents", primary: true },
      { id: "f-sync", location: "/Synthetic/Cloud/Documents", primary: false },
    ],
    offlineFolders: [],
    lastScanAt: new Date().toISOString(),
    operations: [],
    aiEnabled: true,
    addNewFilesWhileScanning: true,
    revision: 1,
  },
};

/** Changes the mocked session state and pushes it like Rust does. */
export function updateMockSession(change: (state: SessionState_Serialize) => SessionState_Serialize) {
  mockSession.state = change(mockSession.state);
  void emit("session-state-event", { state: mockSession.state });
}

let logged: { at: string; text: string; isError: boolean }[] = [];

function log(text: string, isError = false) {
  const entry = { at: new Date().toISOString(), text, isError };
  logged = [...logged, entry];
  void emit("log-event", { entry });
}

/** Runs a fake operation in `lane` with progress, like `Session::perform`. */
export function mockOperation(label: string, lane: "library" | "ai", steps: number, message: string, cancellable = false) {
  if (mockSession.state.operations.some((operation) => operation.lane === lane)) {
    throw { code: "busy", message: `Wait until "${label}" finishes` };
  }
  updateMockSession((state) => ({ ...state, operations: [...state.operations, { label, lane, cancellable }] }));
  log(`${label}…`);
  let done = 0;
  const timer = window.setInterval(() => {
    done++;
    void emit("progress-event", { lanes: [{ lane, progress: { step: label, done, total: steps, detail: `file-${done}.pdf` } }] });
    if (done < steps) return;
    window.clearInterval(timer);
    void emit("progress-event", { lanes: [] });
    log(`${label}: finished in ${(steps * 0.2).toFixed(1)} s`);
    void emit("message-event", { message: { text: message, isError: false } });
    updateMockSession((state) => ({ ...state, operations: state.operations.filter((operation) => operation.lane !== lane), revision: state.revision + 1 }));
  }, 200);
}

export const shellHandlers: Record<string, Handler> = {
  session_state: () => mockSession.state,
  session_progress: () => [],
  log_entries: () => logged,
  clear_log: () => {
    logged = [];
    return null;
  },
  cancel_operation: () => null,
  set_menu_context: () => null,
  file_manager_name: () => "Finder",
  number_locale: () => "en-US",
  open_library: () => null,
  scan_library: () => mockOperation("Scanning the library folders", "library", 12, "Scan finished", true),
  sync_to_all_destinations: () => mockOperation("Syncing to all destinations", "library", 6, "Synced to all destinations: 3 file(s) copied or moved"),
  merge_all_folders: () => mockOperation("Merging all folders", "library", 6, "Merged all folders: 0 file(s) copied or moved"),
  retry_failed_operations: () => mockOperation("Retrying failed operations", "library", 2, "Retried: 0 done, 0 no longer needed, 0 still pending"),
  detect_document_dates: () => mockOperation("Detecting document dates", "library", 3, "Found a date for 4 documents"),
  navigation_counts: () => ({ inbox: 3, tasks: 7, toPay: 1, cases: 2 }),
};
