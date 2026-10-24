import { menuIsOpen } from "./menu";
import { openFolderFlow } from "./folder";
import { openNewProject } from "./newProject";
import { state } from "./store";

const COMBOS: Record<string, () => void> = {
  n: () => openNewProject(),
  o: () => void openFolderFlow(),
  ",": () => {
    location.hash = "#settings";
  },
};

export function installKeys(): void {
  document.addEventListener("keydown", (event) => {
    const mod = event.ctrlKey || event.metaKey;
    const run = COMBOS[event.key.toLowerCase()];
    const blocked = document.querySelector("dialog[open]") || menuIsOpen() || state.opening;
    if (!mod || event.altKey || event.shiftKey || !run || blocked) return;
    event.preventDefault();
    run();
  });
}
