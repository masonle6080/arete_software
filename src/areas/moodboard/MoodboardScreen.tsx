import { lazy, Suspense } from "react";
import type { Moodboard } from "./types";
import "./moodboard.css";
import { EmptyText } from "../../ui";

// Excalidraw is large; load it only when a Moodboard opens.
const MoodboardCanvas = lazy(() => import("./MoodboardCanvas"));

interface Props {
  board: Moodboard;
}

/**
 * One Moodboard on its own screen: its title and the canvas.
 * The canvas saves what is pending when this screen closes.
 */
export function MoodboardScreen({ board }: Props) {
  return (
    <section className="moodboard-screen">
      <h1>{board.title}</h1>
      <Suspense fallback={<EmptyText>Loading canvas…</EmptyText>}>
        <MoodboardCanvas key={board.id} moodboardId={board.id} />
      </Suspense>
    </section>
  );
}
