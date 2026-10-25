import { saveSelected } from "./selection";
import { state, update } from "./store";

export function selectProject(path: string): void {
  saveSelected(path);
  if (state.view === "settings") history.replaceState(null, "", location.pathname);
  update({ selected: path, view: "projects" });
}

export function showProjects(): void {
  history.replaceState(null, "", location.pathname);
  update({ view: "projects" });
}
