import { h } from "./dom";

type Invoke = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

const LABEL = "main";

export function isMac(): boolean {
  return /Mac|iPhone|iPad/.test(navigator.platform);
}

function invoke(cmd: string): void {
  const tauri = (window as unknown as { __TAURI_INTERNALS__?: { invoke?: Invoke } }).__TAURI_INTERNALS__;
  void tauri?.invoke?.(cmd, { label: LABEL });
}

export function closeWindow(): void {
  invoke("plugin:window|close");
}

const BUTTONS = [
  { name: "close", label: "Close", glyph: "×", cmd: "plugin:window|close" },
  { name: "min", label: "Minimize", glyph: "−", cmd: "plugin:window|minimize" },
  { name: "max", label: "Maximize", glyph: "+", cmd: "plugin:window|toggle_maximize" },
];

function control(spec: (typeof BUTTONS)[number]): HTMLButtonElement {
  return h("button", {
    className: `traffic ${spec.name}`, type: "button", "aria-label": spec.label, title: spec.label,
    onClick: () => invoke(spec.cmd),
  }, h("span", { className: "traffic-glyph", "aria-hidden": "true" }, spec.glyph));
}

export function windowControls(): HTMLElement {
  if (isMac()) return h("div", { className: "traffic-space", "aria-hidden": "true" });
  const label = "Window controls";
  return h("div", { className: "traffic-group", role: "group", "aria-label": label }, ...BUTTONS.map(control));
}
