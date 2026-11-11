import {describe, expect, it} from "vitest";
import {afterDrag, clamp, DEFAULT_LAYOUT, panelOf} from "./panels";

describe("panel widths", () => {
  it("clamps the sidebar to 150–360 and the folder tree to 160–480", () => {
    expect(clamp("sidebar", 90)).toBe(150);
    expect(clamp("sidebar", 999)).toBe(360);
    expect(clamp("folderTree", 100)).toBe(160);
  });

  it("folds a panel dragged more than 50 below its narrowest width and keeps the stored width", () => {
    const folded = afterDrag({ ...DEFAULT_LAYOUT, sidebarWidth: 280 }, "sidebar", 99);
    expect(folded.sidebarFolded).toBe(true);
    expect(folded.sidebarWidth).toBe(280);
    const narrow = afterDrag(DEFAULT_LAYOUT, "sidebar", 100);
    expect(narrow.sidebarFolded).toBe(false);
    expect(narrow.sidebarWidth).toBe(150);
  });

  it("shows the dragged width clamped while the drag lasts", () => {
    expect(panelOf({ stored: DEFAULT_LAYOUT, dragged: { folderTree: 9999 } }, "folderTree")).toEqual({ width: 480, folded: false });
    expect(panelOf({ stored: DEFAULT_LAYOUT, dragged: {} }, "sidebar")).toEqual({ width: 200, folded: false });
  });
});
