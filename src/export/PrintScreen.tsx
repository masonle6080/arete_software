import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";
import { toCoreError } from "../shell/coreError";
import { printApi, type PrintJob } from ".";
import "./print.css";

/** Resolves when the fonts and every image in `root` have loaded. */
async function contentReady(root: HTMLElement): Promise<void> {
  await document.fonts.ready;
  const images = Array.from(root.querySelectorAll("img"));
  await Promise.all(
    images
      .filter((img) => !img.complete)
      .map(
        (img) =>
          new Promise((done) => {
            img.addEventListener("load", done, { once: true });
            img.addEventListener("error", done, { once: true });
          }),
      ),
  );
}

/**
 * A job can be taken only once, but React may run an effect twice (Strict
 * Mode). Every caller in this window shares the one take.
 */
const takes = new Map<string, Promise<PrintJob | null>>();
function takeOnce(id: string): Promise<PrintJob | null> {
  if (!takes.has(id)) takes.set(id, printApi.takeJob(id));
  return takes.get(id)!;
}

/**
 * The whole print window: one export, rendered for paper. It opens the print
 * dialog once the content is ready. The toolbar does not print.
 */
export function PrintScreen({ id }: { id: string }) {
  const [job, setJob] = useState<PrintJob | null | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);
  const page = useRef<HTMLElement>(null);

  useEffect(() => {
    takeOnce(id).then(setJob, (e) => setError(toCoreError(e).message));
  }, [id]);

  useEffect(() => {
    if (!job) return;
    document.title = job.title || "Export";
    let cancelled = false;
    contentReady(page.current!).then(() => {
      if (!cancelled) window.print();
    });
    return () => {
      cancelled = true;
    };
  }, [job]);

  const close = () => void getCurrentWindow().close();

  return (
    <div className="print-screen">
      <div className="print-toolbar">
        <button disabled={!job} onClick={() => window.print()}>
          Save as PDF…
        </button>
        <button onClick={close}>Close</button>
        <span className="print-hint">In the print dialog, choose PDF → Save as PDF.</span>
      </div>
      {error && <p className="error">{error}</p>}
      {job === null && <p className="print-missing">This export is no longer available. Export it again.</p>}
      {job?.css && <style>{job.css}</style>}
      {job && <article ref={page} className="print-page" dangerouslySetInnerHTML={{ __html: job.html }} />}
    </div>
  );
}
