import { useState } from "react";
import { MoodboardScreen } from "../moodboard/MoodboardScreen";
import type { Moodboard } from "../moodboard/types";
import { StyleView } from "../style/StyleView";
import { ThesisView, thesisTabs } from "./ThesisView";

interface Props {
  thesisId: number;
  /** The sidebar tab: "overview" or a Thesis section ID. */
  tab: string;
}

/**
 * What a Thesis tab opens on top of itself: a Style of the Drop, or a
 * Moodboard of the Thesis or of a Style (`style` is the Style it returns to).
 */
type Detail = { kind: "style"; id: number } | { kind: "moodboard"; board: Moodboard; style?: number };

/** One tab of the current Thesis, plus the Style and Moodboard screens it opens. */
export function ThesisWorkspace({ thesisId, tab }: Props) {
  const [detail, setDetail] = useState<Detail>();
  const tabTitle = thesisTabs.find((t) => t.id === tab)?.title ?? "Thesis";

  switch (detail?.kind) {
    case "style":
      return (
        <StyleView
          id={detail.id}
          backLabel={tabTitle}
          onBack={() => setDetail(undefined)}
          onOpenStyle={(id) => setDetail({ kind: "style", id })}
          onOpenMoodboard={(board) => setDetail({ kind: "moodboard", board, style: detail.id })}
        />
      );
    case "moodboard": {
      const { style } = detail;
      return (
        <MoodboardScreen
          board={detail.board}
          backLabel={style === undefined ? tabTitle : "Style"}
          onBack={() => setDetail(style === undefined ? undefined : { kind: "style", id: style })}
        />
      );
    }
    default:
      return (
        <ThesisView
          id={thesisId}
          tab={tab}
          onOpenStyle={(id) => setDetail({ kind: "style", id })}
          onOpenMoodboard={(board) => setDetail({ kind: "moodboard", board })}
        />
      );
  }
}
