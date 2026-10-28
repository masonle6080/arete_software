import type { Area } from "../shell/area";
import { essaysArea } from "./essays";
import { libraryArea } from "./library";

/**
 * Navigation: one entry per area, in sidebar order. To add an area, create
 * `src/areas/<area>/index.tsx` exporting an `Area`, then add one import and
 * one line here.
 */
export const areas: Area[] = [
  libraryArea,
  essaysArea,
];
