import { call } from "../../shell/invoke";

/** Mirrors `TechPackSize` in `src-tauri/src/tech_pack/mod.rs`. The ID is stable. */
export interface TechPackSize {
  id: number;
  name: string;
}

/** Mirrors `PointOfMeasure`. The ID is stable across edits. */
export interface PointOfMeasure {
  id: number;
  name: string;
  /** Plus or minus, in the Tech Pack's unit. */
  tolerance: number;
  /** One entry per size, in the order of `TechPack.sizes`; `null` is an empty cell. */
  values: (number | null)[];
}

/** Mirrors `Fabric`. */
export interface Fabric {
  composition: string;
  weight: string;
  color: string;
  vendorId: number | null;
}

/** Mirrors `TechPackDetails`: the written parts of a Tech Pack. */
export interface TechPackDetails {
  unit: string;
  fabric: Fabric;
  trims: string;
  constructionNotes: string;
  labelPlacement: string;
}

/** Mirrors `TechPackFlat`: one flat drawing. */
export interface TechPackFlat {
  id: number;
  media: string;
  fileName: string;
  /** Absolute path of the stored copy in the library. */
  path: string;
}

/** Mirrors `TechPack`. */
export interface TechPack {
  styleId: number;
  /** The columns of the Point of Measure table, and the Style's sizes. */
  sizes: TechPackSize[];
  pointsOfMeasure: PointOfMeasure[];
  details: TechPackDetails;
  flats: TechPackFlat[];
}

/** IPC wrappers for `src-tauri/src/ipc/tech_pack.rs`. */
export const techPackApi = {
  setTechPackBase: (styleId: number) => call<TechPack>("set_tech_pack_base", { styleId }),
  /** `null` when the Style's base is not a Tech Pack. */
  getTechPack: (styleId: number) => call<TechPack | null>("get_tech_pack", { styleId }),
  addSize: (styleId: number, name: string) => call<TechPack>("add_tech_pack_size", { styleId, name }),
  renameSize: (sizeId: number, name: string) => call<TechPack>("rename_tech_pack_size", { sizeId, name }),
  removeSize: (sizeId: number) => call<TechPack>("remove_tech_pack_size", { sizeId }),
  addPointOfMeasure: (styleId: number, name: string, tolerance: number) =>
    call<TechPack>("add_point_of_measure", { styleId, name, tolerance }),
  updatePointOfMeasure: (id: number, name: string, tolerance: number) =>
    call<TechPack>("update_point_of_measure", { id, name, tolerance }),
  setPointOfMeasureValue: (id: number, sizeId: number, value: number | null) =>
    call<TechPack>("set_point_of_measure_value", { id, sizeId, value }),
  removePointOfMeasure: (id: number) => call<TechPack>("remove_point_of_measure", { id }),
  updateDetails: (styleId: number, details: TechPackDetails) =>
    call<TechPack>("update_tech_pack_details", { styleId, details }),
  /** `sources` are paths outside the library; the core imports a copy of each. */
  attachFlats: (styleId: number, sources: string[]) => call<TechPack>("attach_tech_pack_flats", { styleId, sources }),
};
