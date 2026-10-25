import { loadAll } from "./actions";
import { clear } from "./dom";
import { openFolderFlow } from "./folder";
import { installKeys } from "./keys";
import { closeMenu } from "./menu";
import { openNewProject } from "./newProject";
import { shell } from "./projects";
import { onChange, state, update } from "./store";
import { applyTheme } from "./theme";

const app = document.getElementById("app") as HTMLElement;

type Focus = { path: string | null; search: [number, number] | null };

function captureFocus(): Focus {
  const active = document.activeElement as HTMLInputElement | null;
  if (active?.id === "search") return { path: null, search: [active.selectionStart ?? 0, active.selectionEnd ?? 0] };
  return { path: active?.closest("[data-path]")?.getAttribute("data-path") ?? null, search: null };
}

function restoreFocus(focus: Focus): void {
  if (focus.search) {
    const input = document.getElementById("search") as HTMLInputElement | null;
    input?.focus();
    input?.setSelectionRange(focus.search[0], focus.search[1]);
  } else if (focus.path) {
    document.querySelector<HTMLElement>(`[data-path="${CSS.escape(focus.path)}"] button`)?.focus();
  }
}

function render(): void {
  closeMenu();
  const focus = captureFocus();
  clear(app);
  app.append(shell());
  if (state.settings) applyTheme(state.settings.theme);
  restoreFocus(focus);
}

function menuAction(): boolean {
  const action = location.hash;
  if (action !== "#new" && action !== "#open") return false;
  history.replaceState(null, "", location.pathname);
  update({ view: "projects" });
  if (action === "#new") openNewProject();
  else void openFolderFlow();
  return true;
}

function route(): void {
  if (menuAction()) return;
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
