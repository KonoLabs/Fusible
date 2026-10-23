import { Z_UP_PRESETS, type CameraPresetName } from "../cadCamera";
import { demandAuthored, isAuthoredSelected, refreshAuthored, subscribeAuthored, type AuthoredState } from "./authored";
import { buildMechanicalDisplay, poseMechanicalDisplay } from "./display";
import { getMechanicalOption } from "./prefs";
import { framePreset } from "./stage";
import { demandMechanical, subscribeMechanical, type MechanicalModel, type MechanicalState } from "./store";
import { attach, clearAuthored, mount, showAuthored, startLoop, type Tab } from "./tab";

const WAITING = {
  idle: "WAITING FOR CIRCUIT...",
  loading: "GENERATING MECHANICAL MODEL...",
  error: "",
  ready: "",
};

const WAITING_AUTHORED = {
  idle: "SELECT A BODY",
  loading: "LOADING BODY MODEL...",
  error: "",
  ready: "",
};

const LID_BUTTONS = ["btn-mechanical-lid", "btn-mechanical-explode"];

let tab: Tab | null = null;
let generated: MechanicalState = { status: "idle" };
let authored: AuthoredState = { status: "idle" };

function showModel(current: Tab, model: MechanicalModel): void {
  if (current.model === model) return;
  current.display?.dispose();
  current.model = model;
  current.display = buildMechanicalDisplay(model, "solid");
  refreshMechanicalViewer();
}

/** Shows the selected body, its status line, and the demand for its model; hides the lid toggles for authored. */
function sync(): void {
  if (!tab) return;
  const auth = isAuthoredSelected();
  attach(tab, auth ? "authored" : "generated");
  const state = auth ? authored : generated;
  const waiting = auth ? WAITING_AUTHORED : WAITING;
  const text = state.status === "error" ? state.error : waiting[state.status];
  tab.status.textContent = text;
  tab.status.style.display = text ? "flex" : "none";
  demandMechanical("tab", tab.active && !auth);
  demandAuthored("tab", tab.active && auth);
  for (const id of LID_BUTTONS) {
    const button = document.getElementById(id);
    if (button) button.style.display = auth ? "none" : "";
  }
}

function onState(state: MechanicalState): void {
  generated = state;
  if (!tab) return;
  if (state.status === "ready") showModel(tab, state.model);
  if (state.status === "error") {
    tab.display?.dispose();
    tab.display = null;
    tab.model = null;
  }
  sync();
}

function onAuthored(state: AuthoredState): void {
  authored = state;
  if (!tab) return;
  if (state.status === "ready") showAuthored(tab, state.model);
  if (state.status === "error" || state.status === "idle") clearAuthored(tab);
  sync();
}

/** Shows the bodies tab in `container`: the selected authored body, or the generated enclosure. */
export function renderMechanicalViewer(container: HTMLElement): void {
  const first = !tab;
  if (!tab || tab.stage.host !== container) tab = mount(container);
  if (first) {
    subscribeMechanical(onState);
    subscribeAuthored(onAuthored);
  }
  const wasActive = tab.active;
  tab.active = true;
  if (!wasActive) refreshAuthored();
  if (tab.model) showModel(tab, tab.model);
  if (tab.authoredModel) showAuthored(tab, tab.authoredModel);
  sync();
  if (!wasActive) startLoop(() => tab);
}

/** Stops rendering the bodies tab and releases its demand for models. */
export function hideMechanicalViewer(): void {
  if (tab) tab.active = false;
  demandMechanical("tab", false);
  demandAuthored("tab", false);
}

/** Re-applies the lid and explode toggles to the generated enclosure of the tab. */
export function refreshMechanicalViewer(): void {
  if (!tab?.display || !tab.model) return;
  poseMechanicalDisplay(tab.display, tab.model, getMechanicalOption("lid"), getMechanicalOption("explode"));
}

/** Animates the bodies tab camera to a preset, framing the shown body. */
export function mechanicalCameraPreset(preset: CameraPresetName): void {
  if (tab?.stage.content.children.length) framePreset(tab.stage, Z_UP_PRESETS[preset], 600);
}
