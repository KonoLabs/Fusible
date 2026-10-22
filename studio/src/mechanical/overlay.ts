import * as THREE from "three";
import { mainCapturedCanvas } from "../capture";
import { buildMechanicalDisplay, poseMechanicalDisplay, type MechanicalDisplay } from "./display";
import { getMechanicalOption, setMechanicalOption } from "./prefs";
import { demandMechanical, subscribeMechanical, type MechanicalModel, type MechanicalState } from "./store";

type Engine = "cad" | "native";

interface Host {
  display: MechanicalDisplay | null;
  model: MechanicalModel | null;
}

const hosts: Record<Engine, Host> = { cad: { display: null, model: null }, native: { display: null, model: null } };
let cadContainer: HTMLElement | null = null;
let nativeScene: THREE.Scene | null = null;
let current: MechanicalModel | null = null;
let timer = 0;
let shown = false;

function engineScene(engine: Engine): THREE.Scene | null {
  if (engine === "native") return nativeScene;
  return mainCapturedCanvas(cadContainer)?.__threeScene ?? null;
}

function engineMatrix(engine: Engine, model: MechanicalModel): THREE.Matrix4 {
  const mid = model.meta.board_z + model.meta.board_thickness / 2;
  if (engine === "cad") return new THREE.Matrix4().makeTranslation(0, 0, -mid);
  return new THREE.Matrix4().set(1, 0, 0, 0, 0, 0, 1, -mid, 0, 1, 0, 0, 0, 0, 0, 1);
}

function release(host: Host): void {
  host.display?.dispose();
  host.display = null;
  host.model = null;
}

function attach(engine: Engine, model: MechanicalModel, scene: THREE.Scene): void {
  const host = hosts[engine];
  if (host.model !== model) {
    release(host);
    host.model = model;
    host.display = buildMechanicalDisplay(model, "overlay");
    host.display.root.matrixAutoUpdate = false;
    host.display.root.matrix.copy(engineMatrix(engine, model));
  }
  if (!host.display) return;
  poseMechanicalDisplay(host.display, model, getMechanicalOption("lid"), false);
  if (host.display.root.parent !== scene) scene.add(host.display.root);
}

/** Re-attaches, re-poses or removes the overlay in each engine to match the current scenes and toggles. */
export function reconcileMechanicalOverlay(): void {
  for (const engine of Object.keys(hosts) as Engine[]) {
    const scene = engineScene(engine);
    if (shown && current && scene) attach(engine, current, scene);
    else release(hosts[engine]);
  }
}

function onState(state: MechanicalState): void {
  current = state.status === "ready" ? state.model : state.status === "loading" ? current : null;
  reconcileMechanicalOverlay();
}

/**
 * Persists the overlay toggle (`enabled`) and shows the translucent model in both 3D engines while the 3D view is
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

/** Registers the native engine scene so the overlay can follow it. */
export function setMechanicalNativeScene(scene: THREE.Scene | null): void {
  nativeScene = scene;
  reconcileMechanicalOverlay();
}

subscribeMechanical(onState);
