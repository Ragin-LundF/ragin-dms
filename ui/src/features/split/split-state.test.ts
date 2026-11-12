import {describe, expect, it} from "vitest";
import {cutsOf, partsOf, type SplitPart, toggledCut} from "./split-state";

const part = (firstPage: number, lastPage: number, title: string | null): SplitPart => ({
  firstPage, lastPage, title, categoryId: null, newCategory: null, categoryLabel: null, correspondent: null, documentDate: null, confidence: title ? 90 : null,
});

describe("split cuts", () => {
  const proposed = [part(0, 1, "Electricity bill"), part(2, 4, "Tax notice"), part(5, 5, "Receipt")];

  it("turn the AI's parts into cuts and back", () => {
    expect(cutsOf(proposed)).toEqual([2, 5]);
    expect(partsOf(6, [2, 5], proposed)).toEqual(proposed);
  });

  it("keep the AI's values for a part that starts where the AI's did and none for a new one", () => {
    const parts = partsOf(6, toggledCut([2, 5], 4), proposed);

    expect(parts.map((entry) => [entry.firstPage, entry.lastPage, entry.title])).toEqual([[0, 1, "Electricity bill"], [2, 3, "Tax notice"], [4, 4, null], [5, 5, "Receipt"]]);
  });

  it("merge two parts when a cut is removed", () => {
    const parts = partsOf(6, toggledCut([2, 5], 5), proposed);

    expect(parts.map((entry) => [entry.firstPage, entry.lastPage, entry.title])).toEqual([[0, 1, "Electricity bill"], [2, 5, "Tax notice"]]);
  });
});
