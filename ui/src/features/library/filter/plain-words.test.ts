import {describe, expect, it} from "vitest";
import type {FacetFilter} from "../state/view-state";
import {isAiMade, offersPlainWords, withAiFilters} from "./plain-words";

describe("plain words", () => {
  it("are offered from three words on", () => {
    expect(offersPlainWords("unpaid bills")).toBe(false);
    expect(offersPlainWords("  bills from 2025 ")).toBe(true);
  });

  it("replace a facet's filter but add up tags and fields without doubles", () => {
    const current: FacetFilter[] = [{ kind: "year", year: 2024, andEarlier: false }, { kind: "tag", tag: "Tax" }, { kind: "folder", path: "Home" }];
    const added: FacetFilter[] = [{ kind: "year", year: 2025, andEarlier: false }, { kind: "tag", tag: "tax" }, { kind: "tag", tag: "Energy" }];

    expect(withAiFilters(current, added)).toEqual([{ kind: "folder", path: "Home" }, ...added]);
    expect(isAiMade({ kind: "tag", tag: "ENERGY" }, added)).toBe(true);
    expect(isAiMade({ kind: "folder", path: "Home" }, added)).toBe(false);
  });
});
