import type {QueryClient} from "@tanstack/react-query";
import {
    commands,
    events,
    type Lane,
    type LogEntry,
    type OperationProgress_Serialize,
    type SessionState_Serialize,
    type UiMessage
} from "../ipc";
import {createStore, useStore} from "../shared/store";

export type SessionState = SessionState_Serialize;
export type OperationProgress = OperationProgress_Serialize;

/** The shared state as Rust's session last reported it. */
export const sessionStore = createStore<SessionState>({
  status: { kind: "loading" },
  folders: [],
  offlineFolders: [],
  lastScanAt: null,
  operations: [],
  aiEnabled: false,
  addNewFilesWhileScanning: true,
  revision: 0,
});

/** How far the running operations are, by lane. Kept apart: it changes with every file. */
export const progressStore = createStore<Partial<Record<Lane, OperationProgress>>>({});
export const logStore = createStore<LogEntry[]>([]);
/** Messages waiting for the snackbar, oldest first. */
export const messageStore = createStore<UiMessage[]>([]);

const LOG_CAPACITY = 2_000;

/**
 * Every query that reads the library starts its key with this, so a finished operation (a new `revision`)
 * reloads them all, like Kotlin's screens reloading on `revision`.
 */
export const LIBRARY_QUERY = "dms";

export function useSession(): SessionState {
  return useStore(sessionStore);
}

export function useProgress() {
  return useStore(progressStore);
}

/** The label of the operation running in `lane`, or null when the lane is free. */
export function busyIn(state: SessionState, lane: Lane): string | null {
  return state.operations.find((operation) => operation.lane === lane)?.label ?? null;
}

/** Shows a message in the snackbar. */
export function report(message: UiMessage) {
  messageStore.set((queue) => [...queue, message]);
}

export function reportError(error: unknown) {
  report({ text: error instanceof Error ? error.message : String(error), isError: true });
}

/** Loads the current state and follows Rust's events; call once at start. */
export async function connectSession(queries: QueryClient): Promise<void> {
  let revision = sessionStore.get().revision;
  await Promise.all([
    events.sessionStateEvent.listen(({ payload }) => {
      sessionStore.set(payload.state);
      if (payload.state.revision !== revision) {
        revision = payload.state.revision;
        void queries.invalidateQueries({ queryKey: [LIBRARY_QUERY] });
      }
    }),
    events.progressEvent.listen(({ payload }) => {
      progressStore.set(Object.fromEntries(payload.lanes.map((entry) => [entry.lane, entry.progress])));
    }),
    events.logEvent.listen(({ payload }) => {
      logStore.set((entries) => [...entries, payload.entry].slice(-LOG_CAPACITY));
    }),
    events.messageEvent.listen(({ payload }) => report(payload.message)),
  ]);
  const [state, progress, log] = await Promise.all([commands.sessionState(), commands.sessionProgress(), commands.logEntries()]);
  revision = state.revision;
  sessionStore.set(state);
  progressStore.set(Object.fromEntries(progress.map((entry) => [entry.lane, entry.progress])));
  logStore.set(log);
}
