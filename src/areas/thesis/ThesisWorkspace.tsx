import type { ThesisItem } from "../../shell/sidebar";
import { EssayScreen } from "../essays";
import { MoodboardScreen } from "../moodboard/MoodboardScreen";
import { StyleView } from "../style/StyleView";
import { ThesisView, thesisTabs } from "./ThesisView";

interface Props {
  thesisId: number;
  /** The sidebar tab: "overview" or a Thesis section ID. */
  tab: string;
  /** The Style, Essay, or Moodboard open on top of the tab, if any. The shell owns it. */
  open?: ThesisItem;
  /** Opens an item on top of the tab, or returns to the tab with undefined. */
  onOpen(item: ThesisItem | undefined): void;
}

/** One tab of the current Thesis, or the Style, Essay, or Moodboard open on top of it. */
export function ThesisWorkspace({ thesisId, tab, open, onOpen }: Props) {
  const tabTitle = thesisTabs.find((t) => t.id === tab)?.title ?? "Thesis";

  switch (open?.kind) {
    case "style": {
      const styleId = open.id;
      return (
        <StyleView
          key={styleId}
          id={styleId}
          backLabel={tabTitle}
          onBack={() => onOpen(undefined)}
          onOpenStyle={(id) => onOpen({ kind: "style", id })}
          onOpenMoodboard={(board) => onOpen({ kind: "moodboard", board })}
        />
      );
    }
    case "essay":
      return <EssayScreen key={open.id} id={open.id} backLabel={`← ${tabTitle}`} onClose={() => onOpen(undefined)} />;
    case "moodboard": {
      const { owner } = open.board;
      return (
        <MoodboardScreen
          key={open.board.id}
          board={open.board}
          backLabel={owner.kind === "style" ? "Style" : tabTitle}
          onBack={() => onOpen(owner.kind === "style" ? { kind: "style", id: owner.id } : undefined)}
        />
      );
    }
    default:
      return (
        <ThesisView
          id={thesisId}
          tab={tab}
          onOpenStyle={(id) => onOpen({ kind: "style", id })}
          onOpenEssay={(id) => onOpen({ kind: "essay", id })}
          onOpenMoodboard={(board) => onOpen({ kind: "moodboard", board })}
        />
      );
  }
}
