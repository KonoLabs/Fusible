import { loadAll } from "./actions";
import { clear } from "./dom";
import { installKeys } from "./keys";
import { closeMenu } from "./menu";
import { projectsView } from "./projects";
import { settingsView } from "./settings";
import { onChange, state, update } from "./store";
import { applyTheme } from "./theme";

const app = document.getElementById("app") as HTMLElement;

function restoreFocus(path: string | null): void {
  if (!path) return;
  const selector = `[data-path="${CSS.escape(path)}"] button`;
  document.querySelector<HTMLElement>(selector)?.focus();
}

function render(): void {
  closeMenu();
  const focused = (document.activeElement as HTMLElement | null)?.closest("[data-path]");
  const path = focused?.getAttribute("data-path") ?? null;
  clear(app);
  app.append(state.view === "settings" ? settingsView() : projectsView());
  if (state.settings) applyTheme(state.settings.theme);
  restoreFocus(path);
}

function route(): void {
  const view = location.hash === "#settings" ? "settings" : "projects";
  if (view === state.view && state.loaded) return;
  update({ view });
}

onChange(render);
window.addEventListener("hashchange", route);
installKeys();
applyTheme("system");
route();
render();
void loadAll();
