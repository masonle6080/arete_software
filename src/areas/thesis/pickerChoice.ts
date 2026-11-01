/** Option values of the two actions at the end of the Thesis picker. */
export const NEW_THESIS = "new-thesis";
export const ALL_THESES = "all-theses";

export type PickerChoice = { kind: "thesis"; id: number } | { kind: "new" } | { kind: "all" };

/** What a picked option of the Thesis picker means. Undefined for the empty placeholder. */
export function pickerChoice(value: string): PickerChoice | undefined {
  if (value === NEW_THESIS) return { kind: "new" };
  if (value === ALL_THESES) return { kind: "all" };
  const id = Number(value);
  return value !== "" && Number.isInteger(id) ? { kind: "thesis", id } : undefined;
}
