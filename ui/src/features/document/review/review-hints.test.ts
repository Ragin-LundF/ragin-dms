import { describe, expect, it } from "vitest";
import { categoryHint, isNewCategory, shownOptions } from "./review-hints";

const paths = ["Finance", "Finance / Tax", "Home"];

describe("review hints", () => {
  it("a typed category is new unless a path or a last name matches ignoring case", () => {
    expect(isNewCategory("finance / tax", paths)).toBe(false);
    expect(isNewCategory(" TAX ", paths)).toBe(false);
    expect(isNewCategory("Garden", paths)).toBe(true);
    expect(isNewCategory("  ", paths)).toBe(false);
  });

  it("joins the created hint and the rule hint", () => {
    expect(categoryHint(" Garden ", paths, "From rule: Home rule")).toBe('"Garden" will be created · From rule: Home rule');
    expect(categoryHint("Home", paths, null)).toBeNull();
  });

  it("filters options by the typed text unless all are shown", () => {
    expect(shownOptions(paths, "fin", false)).toEqual(["Finance", "Finance / Tax"]);
    expect(shownOptions(paths, "fin", true)).toEqual(paths);
  });
});
