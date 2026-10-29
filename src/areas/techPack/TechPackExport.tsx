import { useState } from "react";
import { exportPdf } from "../../export";
import { toCoreError } from "../../shell/coreError";
import { mediaApi } from "../moodboard/api";
import { vendorApi } from "../vendor/api";
import { techPackApi } from "./api";
import { flatImageSrc, techPackExportCss, techPackExportHtml } from "./export";
import type { Style } from "../style/types";

/** The PDF export button for one Style's Tech Pack. It exports the saved Tech Pack. */
export function TechPackExport({ style }: { style: Style }) {
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function toPdf() {
    setError(null);
    setBusy(true);
    try {
      const techPack = await techPackApi.getTechPack(style.id);
      if (!techPack) throw new Error("This Style is not built on a Tech Pack.");
      const vendorId = techPack.details.fabric.vendorId;
      const [vendors, images] = await Promise.all([
        vendorId === null ? Promise.resolve([]) : vendorApi.listVendors(),
        Promise.all(
          techPack.flats.map(async (f) => {
            try {
              return [f.id, flatImageSrc(f.media, await mediaApi.read(f.media))] as const;
            } catch {
              return null; // The export names a flat it cannot read.
            }
          }),
        ),
      ]);
      const title = `${style.name.trim() || "Untitled"} Tech Pack`;
      await exportPdf({
        title,
        html: techPackExportHtml({
          styleName: style.name,
          styleColor: style.color,
          techPack,
          fabricVendorName: vendors.find((v) => v.id === vendorId)?.name ?? null,
          flatImages: Object.fromEntries(images.filter((e) => e !== null)),
        }),
        css: techPackExportCss,
      });
    } catch (raw) {
      setError(toCoreError(raw).message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="tech-pack-export">
      <button onClick={toPdf} disabled={busy}>
        Export PDF…
      </button>
      <span className="hint">Exports the saved Tech Pack. Save your edits first.</span>
      {error && <span className="error">{error}</span>}
    </div>
  );
}
