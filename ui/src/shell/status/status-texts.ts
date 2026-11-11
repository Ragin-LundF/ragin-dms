import type {SessionState} from "../../session/session";
import {absoluteDate, relativeInstant, timeOfDay} from "../../shared/format/dates";

/** The words of the status bar: the state of the library, how many folders are online, the last scan. */
export function libraryStatusText(state: SessionState, now: Date = new Date()): string {
  const status = {
    loading: "Opening the library",
    notConfigured: "No library yet",
    open: "Library ready",
    primaryFailed: "Library not open",
    failed: "Library not open",
  }[state.status.kind];
  const folders = state.folders.length;
  const offline = state.offlineFolders.length;
  let reach: string | null = null;
  if (folders > 0) {
    if (offline > 0) reach = `${folders} folders, ${offline} offline`;
    else if (folders === 1) reach = "1 folder, online";
    else reach = `${folders} folders, all online`;
  }
  const scanned = state.lastScanAt ? `last scan ${scanTime(state.lastScanAt, now)}` : null;
  return [status, reach, scanned].filter((part) => part !== null).join(" · ");
}

/** The time of day for a scan of today, otherwise how long ago. */
function scanTime(scannedAt: string, now: Date): string {
  if (absoluteDate(scannedAt) === absoluteDate(now)) return timeOfDay(scannedAt);
  return relativeInstant(scannedAt, now);
}
