import { Z_UP_PRESETS } from "../cadCamera";
import type { AuthoredModel } from "./authored";
import { buildAuthoredDisplay, type AuthoredDisplay } from "./authoredDisplay";
import type { MechanicalDisplay } from "./display";
import { framePreset } from "./frame";
import { createStage, resizeStage, type Stage } from "./stage";
import type { MechanicalModel } from "./store";

export interface Tab {
  stage: Stage;
  status: HTMLDivElement;
  display: MechanicalDisplay | null;
  model: MechanicalModel | null;
  authored: AuthoredDisplay | null;
  authoredModel: AuthoredModel | null;
  attached: "generated" | "authored" | null;
  active: boolean;
  framed: boolean;
}

/** Creates the tab: a status line over a fresh three.js stage inside `container`. */
export function mount(container: HTMLElement): Tab {
  container.innerHTML = "";
  const status = document.createElement("div");
  status.className = "empty-state mechanical-status";
  container.appendChild(status);
  return {
    stage: createStage(container),
    status,
    display: null,
    model: null,
    authored: null,
    authoredModel: null,
    attached: null,
    active: false,
    framed: false,
  };
}

/** Builds the display of an authored body into the tab, replacing the previous one. */
export function showAuthored(tab: Tab, model: AuthoredModel): void {
  if (tab.authoredModel === model) return;
  tab.authored?.dispose();
  tab.authoredModel = model;
  tab.authored = buildAuthoredDisplay(model.geometries, model.outline);
}

/** Drops the authored body display of the tab. */
export function clearAuthored(tab: Tab): void {
  tab.authored?.dispose();
  tab.authored = null;
  tab.authoredModel = null;
}

/** Puts the generated or the authored display on the stage, and reframes when the shown body changed. */
export function attach(tab: Tab, which: "generated" | "authored"): void {
  const root = which === "authored" ? tab.authored?.root : tab.display?.root;
  tab.stage.content.clear();
  if (!root) {
    tab.attached = null;
    return;
  }
  tab.stage.content.add(root);
  if (tab.attached !== which) tab.framed = false;
  tab.attached = which;
  ensureFramed(tab);
}

/** Frames the shown content once, as soon as the host has a size. */
export function ensureFramed(tab: Tab): void {
  if (!tab.framed && resizeStage(tab.stage)) {
    tab.framed = true;
    framePreset(tab.stage, Z_UP_PRESETS.ISO, 1);
  }
}

/** Renders frames while the tab returned by `current` is active. */
export function startLoop(current: () => Tab | null): void {
  const tab = current();
  if (!tab?.active) return;
  requestAnimationFrame(() => startLoop(current));
  if (!resizeStage(tab.stage)) return;
  tab.stage.controls.update();
  tab.stage.renderer.render(tab.stage.scene, tab.stage.camera);
}
