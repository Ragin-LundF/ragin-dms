import { describe, expect, it } from "vitest";
import { NO_SELECTION, allOf, chosen, extended, toggled } from "./selection";

const order = ["a", "b", "c", "d", "e"];

describe("document selection", () => {
  it("toggles a tick and moves the anchor", () => {
    const once = toggled(NO_SELECTION, "b");
    expect([...once.selected]).toEqual(["b"]);
    expect(once.anchor).toBe("b");
    expect(toggled(once, "b").selected.size).toBe(0);
  });

  it("extends from the anchor in shown order, both directions, keeping the anchor", () => {
    const ranged = extended(toggled(NO_SELECTION, "d"), "b", order);
    expect([...ranged.selected].sort()).toEqual(["b", "c", "d"]);
    expect(ranged.anchor).toBe("d");
  });

  it("ticks only the clicked document without a shown anchor and makes it the anchor", () => {
    const hiddenAnchor = { selected: new Set(["x"]), anchor: "x" };
    const ranged = extended(hiddenAnchor, "c", order);
    expect([...ranged.selected].sort()).toEqual(["c", "x"]);
    expect(ranged.anchor).toBe("c");
  });

  it("select all clears the anchor and bulk actions only take shown ticks in shown order", () => {
    expect(allOf(order).anchor).toBeNull();
    const state = { selected: new Set(["e", "hidden", "a"]), anchor: null };
    expect(chosen(state, order)).toEqual(["a", "e"]);
  });
});
