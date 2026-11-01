import { describe, expect, it } from "vitest";
import { ALL_THESES, NEW_THESIS, pickerChoice } from "./pickerChoice";

describe("Thesis picker choice", () => {
  it("is a Thesis for a Thesis option", () => {
    expect(pickerChoice("12")).toEqual({ kind: "thesis", id: 12 });
  });

  it("is New Thesis or All Theses for the two options at the end", () => {
    expect(pickerChoice(NEW_THESIS)).toEqual({ kind: "new" });
    expect(pickerChoice(ALL_THESES)).toEqual({ kind: "all" });
  });

  it("is nothing for the empty placeholder", () => {
    expect(pickerChoice("")).toBeUndefined();
  });
});
