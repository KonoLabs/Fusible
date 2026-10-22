import { getMechanicalOption, loadMechanicalOptions, setMechanicalOption, type MechanicalOption } from "./prefs";
import { reconcileMechanicalOverlay, setMechanicalNativeScene, setMechanicalOverlay } from "./overlay";
import { setMechanicalRevision } from "./store";
import { hideMechanicalViewer, mechanicalCameraPreset } from "./viewer";
import { refreshMechanicalViewer, renderMechanicalViewer } from "./viewer";

/** Flips a lid or explode toggle, persists it and re-poses the tab and the overlay. Returns the new value. */
export function toggleMechanicalOption(option: Exclude<MechanicalOption, "overlay">): boolean {
  const next = !getMechanicalOption(option);
  setMechanicalOption(option, next);
  refreshMechanicalViewer();
  reconcileMechanicalOverlay();
  return next;
}

export const Mechanical = {
  renderMechanicalViewer,
  hideMechanicalViewer,
  mechanicalCameraPreset,
  setMechanicalRevision,
  setMechanicalOverlay,
  setMechanicalNativeScene,
  toggleMechanicalOption,
  getMechanicalOptions: loadMechanicalOptions,
};
