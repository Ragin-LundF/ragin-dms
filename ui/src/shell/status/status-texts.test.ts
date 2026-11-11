import {describe, expect, it} from "vitest";
import {libraryStatusText} from "./status-texts";
import type {SessionState} from "../../session/session";

const base: SessionState = {
  status: { kind: "open", foreignFolders: [], unreachableFolders: [] },
  folders: [],
  offlineFolders: [],
  lastScanAt: null,
  operations: [],
  aiEnabled: false,
  addNewFilesWhileScanning: true,
  revision: 0,
};
const folder = (id: string) => ({ id, location: `/tmp/${id}`, primary: id === "a" });

describe("libraryStatusText", () => {
  it("says how many folders are online", () => {
    expect(libraryStatusText({ ...base, folders: [folder("a")] })).toBe("Library ready · 1 folder, online");
    expect(libraryStatusText({ ...base, folders: [folder("a"), folder("b")] })).toBe("Library ready · 2 folders, all online");
    expect(libraryStatusText({ ...base, folders: [folder("a"), folder("b")], offlineFolders: ["b"] })).toBe("Library ready · 2 folders, 1 offline");
  });

  it("shows the time of a scan today and how long ago an older one was", () => {
    const now = new Date(2026, 2, 10, 15, 0);
    expect(libraryStatusText({ ...base, lastScanAt: new Date(2026, 2, 10, 9, 5).toISOString() }, now)).toBe("Library ready · last scan 09:05");
    expect(libraryStatusText({ ...base, lastScanAt: new Date(2026, 2, 7, 9, 5).toISOString() }, now)).toBe("Library ready · last scan 3 days ago");
  });

  it("names a library that is not open", () => {
    expect(libraryStatusText({ ...base, status: { kind: "notConfigured" } })).toBe("No library yet");
    expect(libraryStatusText({ ...base, status: { kind: "failed", detail: null } })).toBe("Library not open");
  });
});
