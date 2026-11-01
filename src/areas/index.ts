import type { Area } from "../shell/area";
import { essaysArea } from "./essays";
import { vendorArea } from "./vendor";
import { blankArea } from "./blank";
import { clipsArea } from "./clips";
import { settingsArea } from "./settings";

/**
 * The sidebar's Shared section: records that exist apart from any one
 * Thesis, in sidebar order. To add one, create `src/areas/<area>/index.tsx`
 * exporting an `Area`, then add one import and one line here. The Thesis
 * area is not an `Area`: the shell centers the sidebar on it (see
 * `src/areas/thesis/index.tsx` and ADR-0004).
 */
export const sharedAreas: Area[] = [essaysArea, vendorArea, blankArea, clipsArea];

export { settingsArea };
