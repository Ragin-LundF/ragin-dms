import {commands, unwrap} from "../ipc";
import {reportError} from "./session";

/** Runs a library-wide operation; it reports its own result as a message, a refused call is shown here. */
function run(operation: () => Promise<{ status: "ok"; data: null } | { status: "error"; error: { code: string; message: string } }>) {
  return () => void unwrap(operation()).catch(reportError);
}

export const operations = {
  openLibrary: run(commands.openLibrary),
  scan: run(commands.scanLibrary),
  syncToAllDestinations: run(commands.syncToAllDestinations),
  mergeAllFolders: run(commands.mergeAllFolders),
  retryFailedOperations: run(commands.retryFailedOperations),
  detectDocumentDates: run(commands.detectDocumentDates),
  checkArchive: run(commands.healthCheckNow),
};
