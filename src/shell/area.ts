import type { ComponentType } from "react";
import type { LibraryInfo } from "../areas/library/session";
import type { Step } from "./navPath";

/** Props every area screen receives from the shell. */
export interface AreaProps {
  library: LibraryInfo;
  /** Opens an item one level down in the navigation path, for example an Essay from the Essays list. */
  onOpen(step: Step): void;
}

/** One sidebar entry and its screen. */
export interface Area {
  id: string;
  label: string;
  Screen: ComponentType<AreaProps>;
}
