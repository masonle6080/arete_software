import type { ComponentType } from "react";
import type { Step } from "../../../shell/navPath";
import type { Style } from "../types";
import type { SegmentPlacement } from "../segments";
import { filesSection } from "./FilesSection";
import { samplesSection } from "./SamplesSection";
import { baseSection } from "./BaseSection";
import { techPackSection } from "./TechPackSection";
import { moodboardSection } from "./MoodboardSection";
import { designDecisionsSection } from "./DesignDecisionsSection";
import { decorationsSection } from "./DecorationsSection";
import { costSheetSection } from "./CostSheetSection";

/** Props every Style view section receives. */
export interface StyleSectionProps {
  style: Style;
  /** Call after a section changes the Style itself (for example, its status). */
  onStyleChanged(): void;
  /** Opens an item one level below the Style in the navigation path, for example its Moodboard. */
  onOpen(step: Step): void;
}

/**
 * One section of the Style view, for example Design files or Cost Sheet. Its
 * `segment` (and, in Build, its `baseKind`) says where the view shows it.
 */
export interface StyleSection extends SegmentPlacement {
  id: string;
  title: string;
  Component: ComponentType<StyleSectionProps>;
}

/**
 * The sections of the Style view, in display order within each segment. To
 * add one, create `sections/<Name>Section.tsx` that exports a `StyleSection`
 * with its `segment`, then add one import and one line here. The view renders
 * the sections of the open segment under their titles; each section owns its
 * own data.
 */
export const styleSections: StyleSection[] = [
  designDecisionsSection,
  decorationsSection,
  moodboardSection,
  filesSection,
  baseSection,
  techPackSection,
  samplesSection,
  costSheetSection,
];
