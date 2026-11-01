import { ScriptScreen } from "../areas/documentary/ScriptScreen";
import { EssayScreen } from "../areas/essays";
import { MoodboardScreen } from "../areas/moodboard/MoodboardScreen";
import { PassageView } from "../areas/passage/PassageView";
import { StyleView } from "../areas/style/StyleView";
import { ThesisView } from "../areas/thesis";
import { steps, type Place, type Step } from "./navPath";

interface Props {
  /** The place that owns the screen (see `screenLevel`). Not a Thesis list or an area. */
  place: Place;
  /** The Passage open over an Essay or a Script, if any. */
  passageId?: string;
  /** The current Thesis, for a Thesis tab. */
  thesisId?: number;
  /** Opens an item one level down. */
  onPush(step: Step): void;
  /** Shows another item in place of this one, for example a Style's new copy. */
  onReplace(step: Step): void;
  /** Changes the breadcrumb label of this place. */
  onRelabel(label: string): void;
}

/** The screen of one place of the navigation path: a Thesis tab, a Style, an Essay, a Moodboard, or a Script. */
export function PlaceScreen({ place, passageId, thesisId, onPush, onReplace, onRelabel }: Props) {
  const openPassage = (id: string) => onPush(steps.passage(id));
  switch (place.kind) {
    case "thesis":
      return thesisId === undefined ? null : <ThesisView id={thesisId} tab={place.tab} onOpen={onPush} />;
    case "style":
      return <StyleView id={place.id} onOpen={onPush} onOpenStyle={onReplace} />;
    case "essay":
      return (
        <EssayScreen
          id={place.id}
          focusPassageId={place.focusPassage}
          passageId={passageId}
          onOpenPassage={openPassage}
          onTitle={(title) => onRelabel(steps.essay(place.id, title).label)}
        />
      );
    case "script":
      return <ScriptScreen documentaryId={place.documentaryId} passageId={passageId} onOpenPassage={openPassage} />;
    case "moodboard":
      return <MoodboardScreen board={place.board} />;
    case "passage":
      return <PassageView id={place.id} />;
    case "theses":
    case "area":
      return null;
  }
}
