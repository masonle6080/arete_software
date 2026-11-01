import { lazy, Suspense } from "react";
import type { Step } from "../../shell/navPath";
import type { Moodboard } from "./types";
import { MoodboardOwnerChip } from "./MoodboardOwnerChip";
import "./moodboard.css";
import { EmptyText } from "../../ui";

// Excalidraw is large; load it only when a Moodboard opens.
const MoodboardCanvas = lazy(() => import("./MoodboardCanvas"));

interface Props {
  board: Moodboard;
  /** Opens what the header chip names: the Thesis or Style the Moodboard belongs to. */
  onOpen(step: Step): void;
}

/**
 * One Moodboard on its own screen: its title, a chip for its owner, and the
 * canvas. The canvas saves what is pending when this screen closes.
 */
export function MoodboardScreen({ board, onOpen }: Props) {
  return (
    <section className="moodboard-screen">
      <h1>{board.title}</h1>
      <div className="chips header-chips">
        <MoodboardOwnerChip owner={board.owner} onOpen={onOpen} />
      </div>
      <Suspense fallback={<EmptyText>Loading canvas…</EmptyText>}>
        <MoodboardCanvas key={board.id} moodboardId={board.id} />
      </Suspense>
    </section>
  );
}
