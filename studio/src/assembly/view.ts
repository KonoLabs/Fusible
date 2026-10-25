import { Z_UP_PRESETS, type CameraPresetName } from "../cadCamera";
import { framePreset, frameSphere } from "../mechanical/frame";
import { errorMessage } from "../mechanical/gltf";
import { createStage } from "../mechanical/stage";
import {
  disposePreview,
  fetchOverview,
  loadPreview,
  violationSphere,
  type Overview,
  type Preview,
  type Violation,
} from "./model";
import { ensureFramed, loop, type View } from "./frameLoop";
import { markPicked, mountPanel, setReport, setStatus } from "./panel";

const MIN_FOCUS_RADIUS = 15;
let view: View | null = null;
let ticket = 0;

function pick(violation: Violation): void {
  if (!view?.preview) return;
  const index = view.overview?.report?.violations.indexOf(violation) ?? -1;
  markPicked(view.panel, index);
  const sphere = violationSphere(violation, view.preview);
  if (!sphere) return;
  sphere.radius = Math.max(sphere.radius, MIN_FOCUS_RADIUS);
  frameSphere(view.stage, Z_UP_PRESETS.ISO, sphere, 600);
}

function show(current: View, overview: Overview, preview: Preview | null): void {
  if (preview && preview !== current.preview) {
    if (current.preview) disposePreview(current.preview);
    current.stage.content.clear();
    current.stage.content.add(preview.root);
    current.preview = preview;
    current.framed = false;
  }
  current.overview = overview;
  setReport(current.panel, overview.report, pick);
  setStatus(current.panel, "");
  ensureFramed(current);
}

function empty(current: View, overview: Overview): void {
  current.overview = overview;
  setReport(current.panel, null, pick);
  if (!overview.declared) return setStatus(current.panel, "NO ASSEMBLY IN fusible.yml");
  setStatus(current.panel, "NO ASSEMBLY PREVIEW YET", true);
}

async function reload(): Promise<void> {
  const current = view;
  if (!current) return;
  const mine = ++ticket;
  try {
    const overview = await fetchOverview();
    const url = overview.model;
    const preview = url && url === current.preview?.url ? current.preview : url ? await loadPreview(url) : null;
    if (mine !== ticket) return;
    if (preview) show(current, overview, preview);
    else empty(current, overview);
  } catch (reason) {
    if (mine === ticket) setStatus(current.panel, errorMessage(reason), true);
  }
}

/** Shows the assembly tab in `container` and loads the latest preview. */
export function renderAssemblyViewer(container: HTMLElement): void {
  if (!view || view.stage.host !== container) {
    container.replaceChildren();
    view = {
      stage: createStage(container),
      panel: mountPanel(container),
      preview: null,
      overview: null,
      active: false,
      framed: false,
    };
  }
  const wasActive = view.active;
  view.active = true;
  void reload();
  if (!wasActive) loop(() => view);
}

/** Stops rendering the assembly tab. */
export function hideAssemblyViewer(): void {
  if (view) view.active = false;
}

/** Reloads the preview and report when the tab is showing, called after an assembly run ends. */
export function refreshAssembly(): void {
  if (view?.active) void reload();
}

/** Animates the assembly camera to a preset, framing the whole preview. */
export function assemblyCameraPreset(preset: CameraPresetName): void {
  if (view?.preview) framePreset(view.stage, Z_UP_PRESETS[preset], 600);
}
