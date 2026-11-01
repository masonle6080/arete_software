import type { Area } from "../shell/area";
import { essaysArea } from "./essays";
import { libraryArea } from "./library";
import { vendorArea } from "./vendor";
import { blankArea } from "./blank";
import { clipsArea } from "./clips";
import { settingsArea } from "./settings";

/**
 * Navigation outside the current Thesis: the library-wide areas, in the
 * order of the sidebar's Library group. To add an area, create
 * `src/areas/<area>/index.tsx` exporting an `Area`, then add one import and
 * one line here. The Thesis area is not an `Area`: the shell centers the
 * sidebar on it (see `src/areas/thesis/index.tsx`).
 */
export const libraryAreas: Area[] = [libraryArea, essaysArea, vendorArea, blankArea, clipsArea];

export { settingsArea };
