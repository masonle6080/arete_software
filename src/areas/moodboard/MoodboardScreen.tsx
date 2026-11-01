import { lazy, Suspense } from "react";
import type { Moodboard } from "./types";
import "./moodboard.css";

// Excalidraw is large; load it only when a Moodboard opens.
const MoodboardCanvas = lazy(() => import("./MoodboardCanvas"));

interface Props {
  board: Moodboard;
  /** Label of the screen that `onBack` returns to, for example the Thesis title. */
  backLabel: string;
  onBack(): void;
}

/**
 * One Moodboard on its own screen: a back button, its title, and the canvas.
 * The canvas saves what is pending when this screen closes.
 */
export function MoodboardScreen({ board, backLabel, onBack }: Props) {
  return (
    <section className="moodboard-screen">
      <button className="back" onClick={onBack}>
        ← {backLabel}
      </button>
      <h1>{board.title}</h1>
      <Suspense fallback={<p className="empty">Loading canvas…</p>}>
        <MoodboardCanvas key={board.id} moodboardId={board.id} />
      </Suspense>
    </section>
  );
}
