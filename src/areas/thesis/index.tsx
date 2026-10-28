import { useState } from "react";
import type { Area } from "../../shell/area";
import { ThesisList } from "./ThesisList";
import { ThesisView } from "./ThesisView";
import "./thesis.css";

function ThesisScreen() {
  const [openId, setOpenId] = useState<number | null>(null);
  return openId === null ? (
    <ThesisList onOpen={setOpenId} />
  ) : (
    <ThesisView id={openId} onBack={() => setOpenId(null)} />
  );
}

export const thesisArea: Area = { id: "thesis", label: "Theses", Screen: ThesisScreen };
