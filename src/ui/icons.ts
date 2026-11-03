/**
 * The one icon set of the app (Lucide). The sidebar rows and the page labels
 * take their icons from here, so the same kind of page always has the same
 * icon.
 */
import {
  Clapperboard,
  Factory,
  FileText,
  Film,
  Images,
  LayoutDashboard,
  Palette,
  Settings,
  Shirt,
  Store,
  type LucideIcon,
} from "lucide-react";

export type Icon = LucideIcon;

/** The icon of each kind of page: the Thesis tabs, the Shared areas, and Settings. */
export const icons = {
  overview: LayoutDashboard,
  essays: FileText,
  moodboards: Images,
  styles: Palette,
  documentary: Clapperboard,
  production: Factory,
  vendor: Store,
  blank: Shirt,
  clips: Film,
  settings: Settings,
} satisfies Record<string, Icon>;
