import {useQuery} from "@tanstack/react-query";
import {
    commands,
    type DashboardAction,
    type DashboardButtonDto_Serialize,
    type DashboardViewDto_Serialize,
    IpcFailure,
    type TaskGroupKind,
    unwrap,
} from "../../ipc";
import {busyIn, LIBRARY_QUERY, reportError, type SessionState, sessionStore} from "../../session/session";
import {operations} from "../../session/operations";
import {dialogs, waitFor} from "../../shared/components";
import {chooseFolder} from "../../shared/platform";
import {createStore, useStore} from "../../shared/store";
import {goTo} from "../../shell/screens";
import {openReview} from "../document";
import {openDocument} from "../library/navigation";
import {openRestructure} from "../restructure/navigation";

export type DashboardView = DashboardViewDto_Serialize;
export type DashboardButton = DashboardButtonDto_Serialize;

/** The group the task list shows, shared by the dashboard and the tasks screen (Kotlin `DashboardViewModel.filter`). */
export const taskFilter = createStore<TaskGroupKind | null>(null);

/** The scans without text found so far; null while nobody looked for them (Kotlin `OrganizeTasks.scans`). */
export const scansStore = createStore<string[] | null>(null);

export function useDashboardView() {
  const scans = useStore(scansStore);
  const scanCount = scans === null ? null : scans.length;
  return useQuery({
    queryKey: [LIBRARY_QUERY, "dashboard", scanCount],
    queryFn: () => unwrap(commands.dashboardView(scanCount)),
  });
}

/** Why `button` cannot be used now: its own reason, or the operation running in the lane it waits for. */
export function disabledReason(button: DashboardButton, session: SessionState): string | null {
  if (button.disabledReason) return button.disabledReason;
  const busy = button.lane ? busyIn(session, button.lane) : null;
  return busy ? waitFor(busy) : null;
}

/** Runs what a button does; asks first when it names a question. */
export async function runButton(button: DashboardButton): Promise<void> {
  if (button.confirm) {
    const { title, message, affected, confirmLabel } = button.confirm;
    if (!(await dialogs.confirm({ title, message, affected, confirmLabel }))) return;
  }
  await runAction(button.action);
}

/** Navigating and opening stay in the window; everything that changes the library goes to Rust. */
export async function runAction(action: DashboardAction): Promise<void> {
  switch (action.kind) {
    case "review":
      return openReview(action.documentId);
    case "open":
      return unwrap(commands.openTaskFile(action.folderId, action.path)).then(() => undefined, reportError);
    case "openDocument":
      return openDocument(action.documentId);
    case "openInbox":
      return goTo("inbox");
    case "restructure":
      return openRestructure(action.folder);
    case "retry":
      return operations.retryFailedOperations();
    case "findScans":
      return findScans();
    case "exportCalendar":
      return exportCalendar();
    case "makeScansSearchable":
      return makeScansSearchable();
    default:
      return unwrap(commands.runDashboardAction(action)).then(() => undefined, reportUnlessBusy);
  }
}

/** Looks for PDFs without a text layer; it only reads. */
export async function findScans(): Promise<void> {
  try {
    scansStore.set(await unwrap(commands.findScansWithoutText()));
  } catch (error) {
    reportError(error);
  }
}

async function exportCalendar(): Promise<void> {
  const directory = await chooseFolder("Choose where to save the calendar");
  if (!directory) return;
  await unwrap(commands.exportCalendar(directory)).catch(reportUnlessBusy);
}

const MAKING_SEARCHABLE = "Making scans searchable";

/** Starts the pdf area's run, and looks for scans again once it ended, as Kotlin did after the run. */
async function makeScansSearchable(): Promise<void> {
  try {
    await unwrap(commands.makeAllSearchable(false));
  } catch (error) {
    reportUnlessBusy(error);
    return;
  }
  const running = () => sessionStore.get().operations.some((operation) => operation.label === MAKING_SEARCHABLE);
  let seen = running();
  const unsubscribe = sessionStore.subscribe(() => {
    if (running()) {
      seen = true;
      return;
    }
    if (!seen) return;
    unsubscribe();
    void findScans();
  });
}

/** A busy lane refuses silently, like Kotlin's session; the button already says why it waits. */
export function reportUnlessBusy(error: unknown) {
  if (error instanceof IpcFailure && error.code === "busy") return;
  reportError(error);
}

/** A year bar's id: "2024", or "≤2019" for that year and all earlier ones (Kotlin `YearFilterModel.parse`). */
export function parseYear(id: string): { year: number; andEarlier: boolean } | null {
  const andEarlier = id.startsWith("≤");
  const year = Number.parseInt(andEarlier ? id.slice(1) : id, 10);
  return Number.isNaN(year) ? null : { year, andEarlier };
}
