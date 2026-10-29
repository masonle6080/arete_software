import { useState } from "react";
import { exportPdf, fileNameFor, saveTextFile } from "../../export";
import { toCoreError } from "../../shell/coreError";
import { designDecisionApi } from "../designDecision/api";
import { styleTraceCss, styleTraceHtml, styleTraceMarkdown, styleTraceTitle } from "./trace";
import type { Style } from "./types";

/** Export buttons for a Style's trace (Style → Design Decisions → Passages): Markdown and PDF. */
export function StyleTraceExport({ style }: { style: Style }) {
  const [message, setMessage] = useState<{ error: boolean; text: string } | null>(null);

  const run = (work: () => Promise<string | null>) => async () => {
    setMessage(null);
    try {
      const text = await work();
      if (text) setMessage({ error: false, text });
    } catch (e) {
      setMessage({ error: true, text: toCoreError(e).message });
    }
  };

  const title = styleTraceTitle(style);

  const toMarkdown = run(async () => {
    const decisions = await designDecisionApi.ofStyle(style.id);
    const path = await saveTextFile({
      defaultName: fileNameFor(title, "md"),
      contents: styleTraceMarkdown(style, decisions),
      filterName: "Markdown",
      extensions: ["md"],
    });
    return path && `Saved ${path}`;
  });

  const toPdf = run(async () => {
    const decisions = await designDecisionApi.ofStyle(style.id);
    await exportPdf({ title, html: styleTraceHtml(style, decisions), css: styleTraceCss });
    return null;
  });

  return (
    <div className="style-trace-export">
      <span className="muted">Export trace</span>
      <button onClick={toMarkdown}>Markdown…</button>
      <button onClick={toPdf}>PDF…</button>
      {message && <span className={message.error ? "error" : "muted"}>{message.text}</span>}
    </div>
  );
}
