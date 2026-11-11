import {describe, expect, it} from "vitest";
import {tagSuggestions} from "./tag-suggestions";

describe("tag suggestions", () => {
  const library = ["car", "Cards", "carpet", "cash", "casino", "cat", "catalog", "invoice"];

  it("offers at most six library tags starting with the typed text, without the document's tags", () => {
    expect(tagSuggestions(" CA", ["Car"], library)).toEqual(["Cards", "carpet", "cash", "casino", "cat", "catalog"]);
    expect(tagSuggestions("inv", [], library)).toEqual(["invoice"]);
  });

  it("offers nothing for empty input", () => {
    expect(tagSuggestions("  ", [], library)).toEqual([]);
  });
});
