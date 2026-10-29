import type { ComponentType } from "react";
import type { LibraryInfo } from "../areas/library/session";

/** Props every area screen receives from the shell. */
export interface AreaProps {
  library: LibraryInfo;
}

/** One sidebar entry and its screen. */
export interface Area {
  id: string;
  label: string;
  Screen: ComponentType<AreaProps>;
}
