import "./thesis.css";

/**
 * The Thesis area. The shell centers its navigation on one current Thesis:
 * the sidebar picks it (`ThesisPicker`) and opens its tabs (`thesisTabs`),
 * and `ThesisWorkspace` shows the open tab. `ThesisList` shows every Thesis.
 */
export { loadCurrentThesis, saveCurrentThesis } from "./currentThesis";
export { ThesisList } from "./ThesisList";
export { ThesisPicker } from "./ThesisPicker";
export { thesisTabs } from "./ThesisView";
export { ThesisWorkspace } from "./ThesisWorkspace";
