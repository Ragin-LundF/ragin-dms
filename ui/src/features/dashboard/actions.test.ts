import {describe, expect, it} from "vitest";
import type {SessionState} from "../../session/session";
import {type DashboardButton, disabledReason, parseYear} from "./actions";

const adopt: DashboardButton = { label: "Adopt", icon: null, style: "tonal", disabledReason: null, lane: "library", action: { kind: "adoptAll" }, confirm: null };
const running = (lane: "library" | "ai"): SessionState => ({
  status: { kind: "open", foreignFolders: [], unreachableFolders: [] },
  folders: [],
  offlineFolders: [],
  lastScanAt: null,
  operations: [{ label: "Scanning the library folders", lane, cancellable: true }],
  aiEnabled: true,
  addNewFilesWhileScanning: true,
  revision: 1,
});

describe("disabledReason", () => {
  it("waits for the operation of the button's lane only", () => {
    expect(disabledReason(adopt, running("library"))).toBe('Wait until "Scanning the library folders" finishes');
    expect(disabledReason(adopt, running("ai"))).toBeNull();
    expect(disabledReason({ ...adopt, lane: null }, running("library"))).toBeNull();
  });

  it("keeps the button's own reason first", () => {
    const unread = { ...adopt, disabledReason: "The file was not read yet; scan again" };
    expect(disabledReason(unread, running("library"))).toBe("The file was not read yet; scan again");
  });
});

describe("parseYear", () => {
  it("reads a folded first bar as that year and earlier", () => {
    expect(parseYear("2024")).toEqual({ year: 2024, andEarlier: false });
    expect(parseYear("≤2019")).toEqual({ year: 2019, andEarlier: true });
    expect(parseYear("")).toBeNull();
  });
});
