import "./thesis.css";

/**
 * The Thesis area. The shell centers its navigation on one current Thesis:
 * the sidebar picks it (`ThesisPicker`) and shows its tabs as flat rows,
 * and `ThesisView` shows the open tab. `ThesisList` shows every Thesis.
 */
export { loadCurrentThesis, saveCurrentThesis } from "./currentThesis";
export { ThesisList } from "./ThesisList";
export { ThesisPicker } from "./ThesisPicker";
export { ThesisView } from "./ThesisView";
