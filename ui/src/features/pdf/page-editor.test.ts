import { describe, expect, it } from "vitest";
import { editSummary, moved, turned } from "./page-editor";

const page = (source: number) => ({ source, rotation: 0, deleted: false, splitAfter: false });

describe("page editor", () => {
  it("turns pages within a full turn", () => {
    expect(turned(page(0), -90).rotation).toBe(270);
    expect(turned({ ...page(0), rotation: 270 }, 90).rotation).toBe(0);
  });

  it("moves a page one place and keeps the ends", () => {
    expect(moved([page(0), page(1), page(2)], 0, 1).map((p) => p.source)).toEqual([1, 0, 2]);
    expect(moved([page(0), page(1)], 0, -1).map((p) => p.source)).toEqual([0, 1]);
  });

  it("counts splits on all but the last page and sees reordering as an edit", () => {
    const pages = [{ ...page(1), splitAfter: true }, page(0), { ...page(2), splitAfter: true }];
    expect(editSummary(pages)).toEqual({ kept: 3, edited: true, splits: 1 });
    expect(editSummary([page(0), page(1)])).toEqual({ kept: 2, edited: false, splits: 0 });
  });
});
