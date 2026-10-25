import { Z_UP_PRESETS } from "../cadCamera";
import { framePreset } from "../mechanical/frame";
import { resizeStage, type Stage } from "../mechanical/stage";
import type { Overview, Preview } from "./model";
import { disposePreview, recolourCollisions } from "./paint";
import type { Panel } from "./panel";

export interface View {
  stage: Stage;
  panel: Panel;
  preview: Preview | null;
  overview: Overview | null;
  active: boolean;
  framed: boolean;
}

/** Frames the preview once, as soon as the host has a size. */
export function ensureFramed(current: View): void {
  if (current.framed || !current.preview || !resizeStage(current.stage)) return;
  current.framed = true;
  framePreset(current.stage, Z_UP_PRESETS.ISO, 1);
}

let latestLoop = 0;

/** Renders frames for as long as the view returned by `current` is active; a newer call retires older loops. */
export function loop(current: () => View | null): void {
  tick(current, ++latestLoop);
}

function tick(current: () => View | null, id: number): void {
  const view = current();
  if (id !== latestLoop || !view?.active) return;
  requestAnimationFrame(() => tick(current, id));
  if (!resizeStage(view.stage)) return;
  ensureFramed(view);
  if (view.preview) recolourCollisions(view.preview);
  view.stage.controls.update();
  view.stage.renderer.render(view.stage.scene, view.stage.camera);
}

/** Disposes the preview of `current` and empties the stage. */
export function clearPreview(current: View): void {
  if (current.preview) disposePreview(current.preview);
  current.stage.content.clear();
  current.preview = null;
  current.framed = false;
}
