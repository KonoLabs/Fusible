import React, { useState, useEffect, useRef } from "react";
import { createRoot, Root } from "react-dom/client";
import * as THREE from "three";
import { OrbitControls } from "three-stdlib";
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

if (typeof window !== "undefined") {
  (window as any).THREE = Object.assign({}, THREE, { OrbitControls });
}

export const FUSIBLE_SCHEMATIC_PALETTE = {
  background: "#000000",
  component_body: "#1F1F1F",
  component_outline: "#86A881",
  wire: "#38bdf8",
  wire_crossing: "#7dd3fc",
  junction: "#38bdf8",
  pin: "#86A881",
  pin_name: "#FFFFFF",
  pin_number: "#A8B365",
  reference: "#86A881",
  value: "#FFFFFF",
  label_local: "#FFFFFF",
  label_global: "#f59e0b",
  label_hier: "#c87d32",
  label_background: "#1F1F1F",
  net_name: "#94a3b8",
  bus: "#818cf8",
  bus_junction: "#818cf8",
  grid: "#1F1F1F",
  grid_axes: "#353535",
  fields: "#cbd5e1",
  note: "#93c5fd",
  sheet: "#353535",
  sheet_background: "#000000",
  sheet_fields: "#94a3b8",
  sheet_filename: "#FFFFFF",
  sheet_label: "#FFFFFF",
  sheet_name: "#FFFFFF",
  cursor: "#86A881",
  aux_items: "#64748b",
  erc_error: "#C55454",
  erc_warning: "#f59e0b",
};

export function renderFusibleSchematicSvg(circuitJson: any, options: any = {}) {
  return convertCircuitJsonToSchematicSvg(circuitJson, {
    ...options,
    colorOverrides: {
      schematic: FUSIBLE_SCHEMATIC_PALETTE,
      ...(options?.colorOverrides || {}),
    },
  });
}

function resolveComponentRefdes(circuitJson: any[], id: string): string {
  if (!id) return "";
  if (!Array.isArray(circuitJson)) return id;
  const directSource = circuitJson.find(
    (e: any) => e.type === "source_component" && (e.name === id || e.source_component_id === id)
  );
  if (directSource?.name) return directSource.name;

  const schComp = circuitJson.find(
    (e: any) => e.type === "schematic_component" && e.schematic_component_id === id
  );
  if (schComp?.source_component_id) {
    const src = circuitJson.find(
      (e: any) => e.type === "source_component" && e.source_component_id === schComp.source_component_id
    );
    if (src?.name) return src.name;
  }

  const pcbComp = circuitJson.find(
    (e: any) => e.type === "pcb_component" && (e.pcb_component_id === id || e.source_component_id === id)
  );
  if (pcbComp?.source_component_id) {
    const src = circuitJson.find(
      (e: any) => e.type === "source_component" && e.source_component_id === pcbComp.source_component_id
    );
    if (src?.name) return src.name;
  }

  return id;
}

function ResponsivePcbViewer({
  circuitJson,
  onSelectComponent,
}: {
  circuitJson: any;
  onSelectComponent?: (name: string) => void;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [viewerHeight, setViewerHeight] = useState(600);

  useEffect(() => {
    try {
      localStorage.setItem("pcb-viewer:rendering-engine", "canvas");
    } catch {}
    const parent = containerRef.current?.parentElement || containerRef.current;
    if (!parent) return;
    const updateSize = () => {
      const h = parent.clientHeight || window.innerHeight - 52;
      if (h > 50) {
        setViewerHeight(h);
      }
    };
    updateSize();
    const ro = new ResizeObserver(updateSize);
    ro.observe(parent);
    window.addEventListener("resize", updateSize);
    return () => {
      ro.disconnect();
      window.removeEventListener("resize", updateSize);
    };
  }, []);

  useEffect(() => {
    if (!onSelectComponent) return;
    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        for (const node of Array.from(mutation.addedNodes)) {
          if (node instanceof HTMLElement) {
            const menu = node.getAttribute("role") === "menu" ? node : node.querySelector('[role="menu"]');
            if (menu) {
              const buttons = menu.querySelectorAll("button");
              for (const btn of Array.from(buttons)) {
                const text = btn.textContent || "";
                if (text.includes("on Schematic")) {
                  const refdes = text.replace("on Schematic", "").trim();
                  if (refdes) {
                    onSelectComponent(refdes);
                    return;
                  }
                }
              }
            }
          }
        }
      }
    });
    observer.observe(document.body, { childList: true, subtree: true });
    return () => observer.disconnect();
  }, [onSelectComponent]);

  return React.createElement(
    "div",
    {
      ref: containerRef,
      style: { width: "100%", height: "100%", position: "relative", overflow: "hidden" },
    },
    React.createElement(PCBViewer, {
      circuitJson: circuitJson || [],
      height: viewerHeight,
      renderer: "canvas",
      allowEditing: false,
      onViewSchematicComponent: (event: any) => {
        if (event?.refdes && onSelectComponent) {
          onSelectComponent(event.refdes);
        }
      },
    })
  );
}

let active3dRoot: Root | null = null;
let active3dContainer: HTMLElement | null = null;

let activePcbRoot: Root | null = null;
let activePcbContainer: HTMLElement | null = null;

let activeSchematicRoot: Root | null = null;
let activeSchematicContainer: HTMLElement | null = null;

export function render3dViewer(
  container: HTMLElement,
  circuitJson: any,
  onSelectComponent?: (name: string) => void
) {
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
      onViewSchematicComponent: (event: any) => {
        const refdes = event?.refdes || event?.source_component_id;
        if (refdes && onSelectComponent) {
          onSelectComponent(resolveComponentRefdes(circuitJson || [], refdes));
        }
      },
      onCameraControllerReady: (controller: any) => {
        if (!controller) return;
        const wrappedController = {
          ...controller,
          animateToPreset: (preset: string) => {
            if (preset === "TOP" || preset === "Top Down") {
              controller.animateToPreset("Top Down");
              return;
            }
            controller.animateToPreset(preset);
          },
        };
        if (typeof window !== "undefined") {
          (window as any).cadCameraController = wrappedController;
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
    React.createElement(ResponsivePcbViewer, {
      circuitJson: circuitJson || [],
      onSelectComponent,
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
      containerStyle: { width: "100%", height: "100%", backgroundColor: "#000000" },
      colorOverrides: {
        schematic: FUSIBLE_SCHEMATIC_PALETTE,
      },
      onSchematicComponentClicked: (event: any) => {
        const id = event?.schematicComponentId || event?.refdes;
        if (id && onSelectComponent) {
          onSelectComponent(resolveComponentRefdes(circuitJson || [], id));
        }
      },
      onViewPcbComponent: (event: any) => {
        const refdes = event?.refdes;
        if (refdes && onSelectComponent) {
          onSelectComponent(resolveComponentRefdes(circuitJson || [], refdes));
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
  convertCircuitJsonToSchematicSvg: renderFusibleSchematicSvg,
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
