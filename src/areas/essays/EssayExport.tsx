import { useState } from "react";
import { exportHtml, exportMarkdown } from "../../document";
import { exportPdf, fileNameFor, saveTextFile } from "../../export";
import { toCoreError } from "../../shell/coreError";
import { documentApi } from "./api";

interface EssayExportProps {
  essayId: number;
  /** Saves unsaved edits, so the export holds the latest text. */
  flush(): Promise<void>;
}

/** Export buttons for one Essay: Markdown and PDF, both with no Passage markers. */
export function EssayExport({ essayId, flush }: EssayExportProps) {
  const [message, setMessage] = useState<{ error: boolean; text: string } | null>(null);

  const latest = async () => {
    await flush();
    return documentApi.open(essayId);
  };

  const run = (work: () => Promise<string | null>) => async () => {
    setMessage(null);
    try {
      const text = await work();
      if (text) setMessage({ error: false, text });
    } catch (e) {
      setMessage({ error: true, text: toCoreError(e).message });
    }
  };

  const toMarkdown = run(async () => {
    const essay = await latest();
    const path = await saveTextFile({
      defaultName: fileNameFor(essay.title, "md"),
      contents: exportMarkdown(essay.title, essay.markdown),
      filterName: "Markdown",
      extensions: ["md"],
    });
    return path && `Saved ${path}`;
  });

  const toPdf = run(async () => {
    const essay = await latest();
    await exportPdf({ title: essay.title || "Untitled", html: exportHtml(essay.title, essay.markdown) });
    return null;
  });

  return (
    <div className="essay-export">
      <span className="muted">Export</span>
      <button onClick={toMarkdown}>Markdown…</button>
      <button onClick={toPdf}>PDF…</button>
      {message && <span className={message.error ? "error" : "muted"}>{message.text}</span>}
    </div>
  );
}
