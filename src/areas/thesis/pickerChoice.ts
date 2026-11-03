/** Option values of the two actions at the end of the Thesis picker. */
export const NEW_THESIS = "new-thesis";
export const ALL_THESES = "all-theses";

/** The prefix of the option value of a Shared area, for example `area:essays`. */
const AREA = "area:";

/** The option value of the Shared area `id`. */
export const areaOption = (id: string) => `${AREA}${id}`;

export type PickerChoice =
  | { kind: "thesis"; id: number }
  | { kind: "area"; id: string }
  | { kind: "new" }
  | { kind: "all" };

/** What a picked option of the Thesis picker means. Undefined for the empty placeholder. */
export function pickerChoice(value: string): PickerChoice | undefined {
  if (value === NEW_THESIS) return { kind: "new" };
  if (value === ALL_THESES) return { kind: "all" };
  if (value.startsWith(AREA) && value.length > AREA.length) return { kind: "area", id: value.slice(AREA.length) };
  const id = Number(value);
  return value !== "" && Number.isInteger(id) ? { kind: "thesis", id } : undefined;
}
