import { openFolderFlow } from "./folder";
import { menuIsOpen } from "./menu";
import { showProjects } from "./navigate";
import { openNewProject } from "./newProject";
import { state } from "./store";
import { isMac } from "./platform";
import { closeWindow } from "./windowControls";

const COMBOS: Record<string, () => void> = {
  n: () => openNewProject(),
  o: () => void openFolderFlow(),
  ",": () => {
    location.hash = "#settings";
  },
  q: () => closeWindow(),
};

const SHIFT_COMBOS: Record<string, () => void> = {
  o: () => showProjects(),
};

function combo(event: KeyboardEvent): (() => void) | undefined {
  const key = event.key.toLowerCase();
  return event.shiftKey ? SHIFT_COMBOS[key] : COMBOS[key];
}

export function installKeys(): void {
  if (isMac()) return;
  document.addEventListener("keydown", (event) => {
    const mod = event.ctrlKey || event.metaKey;
    const run = combo(event);
    const blocked = document.querySelector("dialog[open]") || menuIsOpen() || state.opening;
    if (!mod || event.altKey || !run || blocked) return;
    event.preventDefault();
    run();
  });
}
