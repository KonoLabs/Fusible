import React from "react";
import { createRoot, Root } from "react-dom/client";
import {
  convertCircuitJsonToSchematicSvg,
  convertCircuitJsonToPcbSvg,
  colorMap,
} from "circuit-to-svg";
import { CadViewer } from "@tscircuit/3d-viewer";

let activeRoot: Root | null = null;
let activeContainer: HTMLElement | null = null;

export function render3dViewer(container: HTMLElement, circuitJson: any) {
  if (!container) return;
  if (activeContainer !== container) {
    if (activeRoot) {
      activeRoot.unmount();
      activeRoot = null;
    }
    container.innerHTML = "";
    activeRoot = createRoot(container);
    activeContainer = container;
  }
  if (!activeRoot) {
    activeRoot = createRoot(container);
    activeContainer = container;
  }
  activeRoot.render(
    React.createElement(CadViewer, {
      circuitJson: circuitJson || [],
      autoRotateDisabled: true,
      clickToInteractEnabled: false,
    })
  );
}

export {
  convertCircuitJsonToSchematicSvg,
  convertCircuitJsonToPcbSvg,
  colorMap,
  CadViewer,
};

export const Tscircuit = {
  convertCircuitJsonToSchematicSvg,
  convertCircuitJsonToPcbSvg,
  colorMap,
  CadViewer,
  render3dViewer,
};

declare global {
  interface Window {
    Tscircuit: typeof Tscircuit;
  }
}

if (typeof window !== "undefined") {
  window.Tscircuit = Tscircuit;
}

export default Tscircuit;
