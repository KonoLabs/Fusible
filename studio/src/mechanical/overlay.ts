import * as THREE from "three";
import { mainCapturedCanvas } from "../capture";
import { buildMechanicalDisplay, poseMechanicalDisplay, type MechanicalDisplay } from "./display";
import { getMechanicalOption, setMechanicalOption } from "./prefs";
import { demandMechanical, subscribeMechanical, type MechanicalModel, type MechanicalState } from "./store";

let host: { display: MechanicalDisplay | null; model: MechanicalModel | null } = { display: null, model: null };
let cadContainer: HTMLElement | null = null;
let current: MechanicalModel | null = null;
let timer = 0;
let shown = false;

function release(): void {
  host.display?.dispose();
  host = { display: null, model: null };
}

function attach(model: MechanicalModel, scene: THREE.Scene): void {
  if (host.model !== model) {
    release();
    const mid = model.meta.board_z + model.meta.board_thickness / 2;
    const display = buildMechanicalDisplay(model, "overlay");
    display.root.matrixAutoUpdate = false;
    display.root.matrix.makeTranslation(0, 0, -mid);
    host = { display, model };
  }
  if (!host.display) return;
  poseMechanicalDisplay(host.display, model, getMechanicalOption("lid"), false);
  if (host.display.root.parent !== scene) scene.add(host.display.root);
}

/** Re-attaches, re-poses or removes the overlay in the CadViewer scene to match the current model and toggles. */
export function reconcileMechanicalOverlay(): void {
  const scene = mainCapturedCanvas(cadContainer)?.__threeScene ?? null;
  if (shown && current && scene) attach(current, scene);
  else release();
}

function onState(state: MechanicalState): void {
  current = state.status === "ready" ? state.model : state.status === "loading" ? current : null;
  reconcileMechanicalOverlay();
}

/**
 * Persists the overlay toggle (`enabled`) and shows the translucent model in the CadViewer while the 3D view is
 * `visible`. `container` hosts the CadViewer. The model is only fetched while the overlay is enabled and visible.
 */
export function setMechanicalOverlay(enabled: boolean, visible: boolean, container: HTMLElement | null): void {
  cadContainer = container ?? cadContainer;
  setMechanicalOption("overlay", enabled);
  const active = enabled && visible;
  if (active !== shown) {
    window.clearInterval(timer);
    timer = active ? window.setInterval(reconcileMechanicalOverlay, 300) : 0;
  }
  shown = active;
  demandMechanical("overlay", active);
  reconcileMechanicalOverlay();
}

subscribeMechanical(onState);
