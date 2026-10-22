import { Z_UP_PRESETS, type CameraPresetName } from "../cadCamera";
import { buildMechanicalDisplay, poseMechanicalDisplay, type MechanicalDisplay } from "./display";
import { getMechanicalOption } from "./prefs";
import { createStage, framePreset, resizeStage, type Stage } from "./stage";
import { demandMechanical, subscribeMechanical, type MechanicalModel, type MechanicalState } from "./store";

interface Tab {
  stage: Stage;
  status: HTMLDivElement;
  display: MechanicalDisplay | null;
  model: MechanicalModel | null;
  active: boolean;
  framed: boolean;
}

const WAITING: Record<MechanicalState["status"], string> = {
  idle: "WAITING FOR CIRCUIT...",
  loading: "GENERATING MECHANICAL MODEL...",
  error: "",
  ready: "",
};

let tab: Tab | null = null;

function showModel(current: Tab, model: MechanicalModel): void {
  if (current.model !== model) {
    current.display?.dispose();
    current.model = model;
    current.display = buildMechanicalDisplay(model, "solid");
    current.stage.content.add(current.display.root);
    refreshMechanicalViewer();
  }
  if (!current.framed && resizeStage(current.stage)) {
    current.framed = true;
    framePreset(current.stage, Z_UP_PRESETS.ISO, 1);
  }
}

function onState(state: MechanicalState): void {
  if (!tab) return;
  if (state.status === "ready") showModel(tab, state.model);
  if (state.status === "error") {
    tab.display?.dispose();
    tab.display = null;
    tab.model = null;
  }
  const text = state.status === "error" ? state.error : WAITING[state.status];
  tab.status.textContent = text;
  tab.status.style.display = text ? "flex" : "none";
}

function loop(): void {
  if (!tab?.active) return;
  requestAnimationFrame(loop);
  if (!resizeStage(tab.stage)) return;
  tab.stage.controls.update();
  tab.stage.renderer.render(tab.stage.scene, tab.stage.camera);
}

function mount(container: HTMLElement): Tab {
  container.innerHTML = "";
  const status = document.createElement("div");
  status.className = "empty-state mechanical-status";
  container.appendChild(status);
  return { stage: createStage(container), status, display: null, model: null, active: false, framed: false };
}

/** Shows the mechanical tab in `container`, fetching the model for the current revision when needed. */
export function renderMechanicalViewer(container: HTMLElement): void {
  const first = !tab;
  if (!tab || tab.stage.host !== container) tab = mount(container);
  if (first) subscribeMechanical(onState);
  const wasActive = tab.active;
  tab.active = true;
  demandMechanical("tab", true);
  if (tab.model) showModel(tab, tab.model);
  if (!wasActive) loop();
}

/** Stops rendering the mechanical tab and releases its demand for the model. */
export function hideMechanicalViewer(): void {
  if (tab) tab.active = false;
  demandMechanical("tab", false);
}

/** Re-applies the lid and explode toggles to the mechanical tab. */
export function refreshMechanicalViewer(): void {
  if (!tab?.display || !tab.model) return;
  poseMechanicalDisplay(tab.display, tab.model, getMechanicalOption("lid"), getMechanicalOption("explode"));
}

/** Animates the mechanical tab camera to a preset, framing the model bounds. */
export function mechanicalCameraPreset(preset: CameraPresetName): void {
  if (tab?.model) framePreset(tab.stage, Z_UP_PRESETS[preset], 600);
}
