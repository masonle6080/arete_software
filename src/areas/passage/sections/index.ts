import type { ComponentType } from "react";
import type { Passage } from "../api";
import { moodboardItemsSection } from "./MoodboardItemsSection";

/** Props every Passage detail section receives. */
export interface PassageSectionProps {
  passage: Passage;
}

/** One kind of record linked to a Passage, for example Design Decisions. */
export interface PassageSection {
  id: string;
  title: string;
  Component: ComponentType<PassageSectionProps>;
}

/**
 * The sections of the Passage detail view, in display order: one per kind
 * of record linked to a Passage. To add one (Design Decisions, Moodboard
 * items, Clips), create `sections/<Name>Section.tsx` that exports a
 * `PassageSection`, then add one import and one line here. The view renders
 * each section under its title; the section owns its own data.
 */
export const passageSections: PassageSection[] = [
  moodboardItemsSection,
];
