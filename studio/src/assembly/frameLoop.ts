import { Z_UP_PRESETS } from "../cadCamera";
import { framePreset } from "../mechanical/frame";
import { resizeStage, type Stage } from "../mechanical/stage";
import type { Overview, Preview } from "./model";
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

/** Renders frames for as long as the view returned by `current` is active. */
export function loop(current: () => View | null): void {
  const view = current();
  if (!view?.active) return;
  requestAnimationFrame(() => loop(current));
  if (!resizeStage(view.stage)) return;
  ensureFramed(view);
  view.stage.controls.update();
  view.stage.renderer.render(view.stage.scene, view.stage.camera);
}
