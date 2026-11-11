import {describe, expect, it} from "vitest";
import {paletteShortcut} from "./palette-actions";
import {moveSelection, paletteGroups} from "./palette-items";

const none = { documents: [], places: [] };

describe("paletteGroups", () => {
  it("lists every action without a query, AI actions only with AI", () => {
    const withoutAi = paletteGroups("", none, {}, false, "⌘");
    expect(withoutAi.map((group) => group.title)).toEqual(["Actions"]);
    expect(withoutAi[0]?.items.some((item) => item.label === "Tag untagged documents with AI")).toBe(false);
  });

  it("says why an action of a busy lane waits and offers to ask the library", () => {
    const groups = paletteGroups("scan", none, { library: "Merging all folders" }, true, "⌘");
    const items = groups[0]?.items ?? [];
    expect(items[0]).toMatchObject({ label: "Scan now", disabledReason: 'Wait until "Merging all folders" finishes', hint: paletteShortcut("scan") });
    expect(items[1]).toMatchObject({ kind: "ask", label: "Ask the library: scan", hint: "⌘↵" });
  });

  it("turns places into library selections", () => {
    const groups = paletteGroups("tax", { documents: [], places: [{ kind: "Category", value: "c-2", label: "Finance / Tax", count: 3 }] }, {}, false, "⌘");
    expect(groups[0]?.items[0]).toMatchObject({ kind: "goTo", selection: { kind: "category", categoryId: "c-2" }, detail: "Category", hint: "3" });
  });
});

describe("moveSelection", () => {
  it("wraps around at both ends", () => {
    expect(moveSelection(0, -1, 3)).toBe(2);
    expect(moveSelection(2, 1, 3)).toBe(0);
    expect(moveSelection(5, 0, 0)).toBe(0);
  });
});
