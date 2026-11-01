import type { StyleBase } from "./types";

/** The three segments of the Style view. */
export type StyleSegment = "design" | "build" | "make";

/** The segments in display order, as `SegmentedControl` options. */
export const styleSegments: readonly { value: StyleSegment; label: string }[] = [
  { value: "design", label: "Design" },
  { value: "build", label: "Build" },
  { value: "make", label: "Make" },
];

/** The kind of base a Build section applies to. */
export type BaseKind = "blank" | "techPack";

/** Where a section shows: its segment and, in Build, the base kind it applies to. */
export interface SegmentPlacement {
  segment: StyleSegment;
  /** Build sections only: the base kind the section applies to. */
  baseKind?: BaseKind;
}

/**
 * The base kind the Build segment opens on: Blank for a Blank Style, Tech
 * Pack otherwise (a Tech Pack Style, or a Style with no base yet).
 */
export function baseKindOf(base: StyleBase | null): BaseKind {
  return base?.kind === "blank" ? "blank" : "techPack";
}

/**
 * The sections one segment shows, in registration order. Build shows only
 * the sections for `baseKind`; Design and Make ignore it.
 */
export function sectionsIn<S extends SegmentPlacement>(sections: readonly S[], segment: StyleSegment, baseKind: BaseKind): S[] {
  return sections.filter(
    (s) => s.segment === segment && (s.baseKind === undefined || s.baseKind === baseKind),
  );
}
