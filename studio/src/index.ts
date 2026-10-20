import React from "react";
import { createRoot, Root } from "react-dom/client";
import {
  convertCircuitJsonToSchematicSvg,
  convertCircuitJsonToPcbSvg,
  colorMap,
} from "circuit-to-svg";
import { CadViewer } from "@tscircuit/3d-viewer";
import { PCBViewer } from "@tscircuit/pcb-viewer";
import { SchematicViewer } from "@tscircuit/schematic-viewer";
import "iconify-icon";
import { addCollection } from "iconify-icon";
import pixelarticons from "@iconify-json/pixelarticons/icons.json";

addCollection(pixelarticons as any);

let active3dRoot: Root | null = null;
let active3dContainer: HTMLElement | null = null;

let activePcbRoot: Root | null = null;
let activePcbContainer: HTMLElement | null = null;

let activeSchematicRoot: Root | null = null;
let activeSchematicContainer: HTMLElement | null = null;

export function render3dViewer(container: HTMLElement, circuitJson: any) {
  if (!container) return;
  if (active3dContainer !== container) {
    if (active3dRoot) {
      active3dRoot.unmount();
      active3dRoot = null;
    }
    container.innerHTML = "";
    active3dRoot = createRoot(container);
    active3dContainer = container;
  }
  if (!active3dRoot) {
    active3dRoot = createRoot(container);
    active3dContainer = container;
  }
  active3dRoot.render(
    React.createElement(CadViewer, {
      circuitJson: circuitJson || [],
      autoRotateDisabled: true,
      clickToInteractEnabled: false,
      onCameraControllerReady: (controller: any) => {
        if (typeof window !== "undefined") {
          (window as any).cadCameraController = controller;
        }
      },
    })
  );
}

export function renderPcbViewer(
  container: HTMLElement,
  circuitJson: any,
  onSelectComponent?: (name: string) => void
) {
  if (!container) return;
  if (activePcbContainer !== container) {
    if (activePcbRoot) {
      activePcbRoot.unmount();
      activePcbRoot = null;
    }
    container.innerHTML = "";
    activePcbRoot = createRoot(container);
    activePcbContainer = container;
  }
  if (!activePcbRoot) {
    activePcbRoot = createRoot(container);
    activePcbContainer = container;
  }
  activePcbRoot.render(
    React.createElement(PCBViewer, {
      circuitJson: circuitJson || [],
      allowEditing: false,
      onViewSchematicComponent: (event: any) => {
        if (event?.refdes && onSelectComponent) {
          onSelectComponent(event.refdes);
        }
      },
    })
  );
}

export function renderSchematicViewer(
  container: HTMLElement,
  circuitJson: any,
  onSelectComponent?: (name: string) => void
) {
  if (!container) return;
  if (activeSchematicContainer !== container) {
    if (activeSchematicRoot) {
      activeSchematicRoot.unmount();
      activeSchematicRoot = null;
    }
    container.innerHTML = "";
    activeSchematicRoot = createRoot(container);
    activeSchematicContainer = container;
  }
  if (!activeSchematicRoot) {
    activeSchematicRoot = createRoot(container);
    activeSchematicContainer = container;
  }
  activeSchematicRoot.render(
    React.createElement(SchematicViewer, {
      circuitJson: circuitJson || [],
      containerStyle: { width: "100%", height: "100%" },
      onSchematicComponentClicked: (event: any) => {
        if (event?.schematicComponentId && onSelectComponent) {
          onSelectComponent(event.schematicComponentId);
        }
      },
      onViewPcbComponent: (event: any) => {
        if (event?.refdes && onSelectComponent) {
          onSelectComponent(event.refdes);
        }
      },
    })
  );
}

export {
  convertCircuitJsonToSchematicSvg,
  convertCircuitJsonToPcbSvg,
  colorMap,
  CadViewer,
  PCBViewer,
  SchematicViewer,
};

export const Tscircuit = {
  convertCircuitJsonToSchematicSvg,
  convertCircuitJsonToPcbSvg,
  colorMap,
  CadViewer,
  PCBViewer,
  SchematicViewer,
  render3dViewer,
  renderPcbViewer,
  renderSchematicViewer,
};

declare global {
  interface Window {
    Tscircuit: typeof Tscircuit;
    cadCameraController?: any;
  }
}

if (typeof window !== "undefined") {
  window.Tscircuit = Tscircuit;
  window.dispatchEvent(new CustomEvent("tscircuit-ready"));
}

export default Tscircuit;
