import { useState } from "react";
import type { Area } from "../../shell/area";
import { StyleView } from "../style/StyleView";
import { ThesisList } from "./ThesisList";
import { ThesisView } from "./ThesisView";
import "./thesis.css";

/** Which screen of the Theses area is open: the list, one Thesis, or one Style of its Drop. */
type Screen = { kind: "list" } | { kind: "thesis"; id: number } | { kind: "style"; thesisId: number; id: number };

function ThesisScreen() {
  const [screen, setScreen] = useState<Screen>({ kind: "list" });
  switch (screen.kind) {
    case "list":
      return <ThesisList onOpen={(id) => setScreen({ kind: "thesis", id })} />;
    case "thesis":
      return (
        <ThesisView
          id={screen.id}
          onBack={() => setScreen({ kind: "list" })}
          onOpenStyle={(styleId) => setScreen({ kind: "style", thesisId: screen.id, id: styleId })}
        />
      );
    case "style":
      return (
        <StyleView
          id={screen.id}
          backLabel="Thesis"
          onBack={() => setScreen({ kind: "thesis", id: screen.thesisId })}
        />
      );
  }
}

export const thesisArea: Area = { id: "thesis", label: "Theses", Screen: ThesisScreen };
