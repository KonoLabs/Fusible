import React, { useState, useEffect, useRef } from "react";
import { createRoot, Root } from "react-dom/client";
import * as THREE from "three";
import { OrbitControls } from "three-stdlib";
import {
  convertCircuitJsonToSchematicSvg,
  convertCircuitJsonToPcbSvg,
  colorMap,
} from "circuit-to-svg";
import { CadViewer as BaseCadViewer } from "@tscircuit/3d-viewer";
import { PCBViewer } from "@tscircuit/pcb-viewer";
import { SchematicViewer } from "@tscircuit/schematic-viewer";
import "iconify-icon";
import { addCollection } from "iconify-icon";
import pixelarticons from "@iconify-json/pixelarticons/icons.json";

addCollection(pixelarticons as any);

if (typeof window !== "undefined") {
  (window as any).THREE = Object.assign({}, THREE, { OrbitControls });
}

if (typeof window !== "undefined" && (THREE as any)?.WebGLRenderer && !(THREE.WebGLRenderer.prototype as any).__fusiblePatched) {
  const origRender = THREE.WebGLRenderer.prototype.render;
  THREE.WebGLRenderer.prototype.render = function (scene: any, camera: any) {
    if (this.domElement) {
      (this.domElement as any).__threeScene = scene;
      (this.domElement as any).__threeCamera = camera;
      (this.domElement as any).__threeRenderer = this;
    }
    (window as any).__lastThreeScene = scene;
    (window as any).__lastThreeCamera = camera;
    return origRender.apply(this, arguments as any);
  };
  (THREE.WebGLRenderer.prototype as any).__fusiblePatched = true;
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
  label_hier: "#A8B365",
  label_background: "#1F1F1F",
  net_name: "#38bdf8",
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

export function normalizeCircuitJsonForSchematic(circuitJson: any) {
  if (!Array.isArray(circuitJson)) return circuitJson || [];
  const sheets = circuitJson.filter((e: any) => e?.type === "schematic_sheet");
  if (!sheets.length) return circuitJson;

  const components = circuitJson.filter((e: any) => e?.type === "schematic_component");
  const componentsWithSheet = components.filter((e: any) => !!e.schematic_sheet_id);

  if (components.length > 0 && componentsWithSheet.length === 0) {
    return circuitJson.filter((e: any) => e?.type !== "schematic_sheet");
  }

  return circuitJson;
}

export function renderFusibleSchematicSvg(circuitJson: any, options: any = {}) {
  const normalized = normalizeCircuitJsonForSchematic(circuitJson);
  return convertCircuitJsonToSchematicSvg(normalized, {
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
    (e: any) => e.type === "schematic_component" && (e.schematic_component_id === id || e.source_component_id === id)
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

  const cadComp = circuitJson.find(
    (e: any) => e.type === "cad_component" && (e.cad_component_id === id || e.source_component_id === id || e.pcb_component_id === id)
  );
  if (cadComp?.source_component_id) {
    const src = circuitJson.find(
      (e: any) => e.type === "source_component" && e.source_component_id === cadComp.source_component_id
    );
    if (src?.name) return src.name;
  }
  if (cadComp?.pcb_component_id) {
    const pcb = circuitJson.find(
      (e: any) => e.type === "pcb_component" && e.pcb_component_id === cadComp.pcb_component_id
    );
    if (pcb?.source_component_id) {
      const src = circuitJson.find(
        (e: any) => e.type === "source_component" && e.source_component_id === pcb.source_component_id
      );
      if (src?.name) return src.name;
    }
  }

  return id;
}

function determineRefdesFrom3dObject(circuitJson: any[], object: any): string {
  let curr: any = object;
  let pcbId = "";
  let cadId = "";
  let srcId = "";
  let refdes = "";
  while (curr) {
    if (curr.visible === false) return "";
    if (curr.userData?.isReferenceObject) return "";
    if (!pcbId && curr.userData?.pcb_component_id) pcbId = curr.userData.pcb_component_id;
    if (!cadId && curr.userData?.cad_component_id) cadId = curr.userData.cad_component_id;
    if (!srcId && curr.userData?.source_component_id) srcId = curr.userData.source_component_id;
    if (!refdes && curr.userData?.refdes) refdes = curr.userData.refdes;
    if (!refdes && curr.userData?.name) refdes = curr.userData.name;
    curr = curr.parent;
  }
  const candidate = srcId || pcbId || cadId || refdes;
  if (!candidate) return "";
  const resolved = resolveComponentRefdes(circuitJson, candidate);
  if (resolved && circuitJson.some((e: any) => e.type === "source_component" && e.name === resolved)) {
    return resolved;
  }
  if (resolved && !resolved.startsWith("cad_") && !resolved.startsWith("pcb_") && !resolved.startsWith("source_") && !resolved.startsWith("schematic_")) {
    return resolved;
  }
  return "";
}

export function CadViewer({
  circuitJson,
  onSelectComponent,
  autoRotateDisabled = true,
  clickToInteractEnabled = false,
  onViewSchematicComponent,
  onCameraControllerReady,
  ...props
}: any) {
  const containerRef = useRef<HTMLDivElement>(null);
  const pointerStartRef = useRef<{ x: number; y: number } | null>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || !onSelectComponent) return;

    const handlePointerDown = (e: PointerEvent) => {
      pointerStartRef.current = { x: e.clientX, y: e.clientY };
    };

    const handleRaycast = (clientX: number, clientY: number) => {
      const canvas = container.querySelector("canvas");
      if (!canvas) return;
      const camera = (canvas as any).__threeCamera || (window as any).__lastThreeCamera;
      const scene = (canvas as any).__threeScene || (window as any).__lastThreeScene || (window as any).__TSCIRCUIT_THREE_OBJECT?.parent;
      if (!camera || !scene) return;

      const rect = canvas.getBoundingClientRect();
      if (!rect.width || !rect.height) return;
      if (
        clientX < rect.left ||
        clientX > rect.right ||
        clientY < rect.top ||
        clientY > rect.bottom
      ) {
        return;
      }

      const mouse = new THREE.Vector2(
        ((clientX - rect.left) / rect.width) * 2 - 1,
        -((clientY - rect.top) / rect.height) * 2 + 1
      );
      camera.updateWorldMatrix(true, false);
      scene.updateWorldMatrix(true, true);
      const raycaster = new THREE.Raycaster();
      raycaster.setFromCamera(mouse, camera);
      const hits = raycaster.intersectObjects(scene.children, true);

      for (const hit of hits) {
        if (!(hit.object instanceof THREE.Mesh)) continue;
        const refdes = determineRefdesFrom3dObject(circuitJson || [], hit.object);
        if (refdes) {
          onSelectComponent(refdes);
          break;
        }
      }
    };

    const handleClick = (e: MouseEvent) => {
      const start = pointerStartRef.current;
      if (start && Math.hypot(e.clientX - start.x, e.clientY - start.y) > 6) {
        return;
      }
      handleRaycast(e.clientX, e.clientY);
    };

    const handleContextMenu = (e: MouseEvent) => {
      handleRaycast(e.clientX, e.clientY);
    };

    container.addEventListener("pointerdown", handlePointerDown, true);
    container.addEventListener("click", handleClick, true);
    container.addEventListener("contextmenu", handleContextMenu, true);

    const handleDocumentClick = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null;
      const btn = target?.closest('button, [role="menuitem"]');
      if (btn) {
        const text = btn.textContent || "";
        if (text.includes("on Schematic")) {
          const refdes = text.replace("on Schematic", "").replace(/[↗︎\s]/g, "").trim();
          if (refdes) {
            const resolved = resolveComponentRefdes(circuitJson || [], refdes);
            if (resolved) onSelectComponent(resolved);
          }
        }
        const match = text.match(/Hide\s*"([^"]+)"/);
        if (match && match[1]) {
          const resolved = resolveComponentRefdes(circuitJson || [], match[1]);
          if (resolved) onSelectComponent(resolved);
        }
      }
    };
    document.addEventListener("click", handleDocumentClick, true);

    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        for (const node of Array.from(mutation.addedNodes)) {
          if (node instanceof HTMLElement) {
            const menu = node.getAttribute("role") === "menu" ? node : node.querySelector('[role="menu"]');
            if (menu) {
              const buttons = menu.querySelectorAll("button, [role=\"menuitem\"]");
              for (const btn of Array.from(buttons)) {
                const text = btn.textContent || "";
                if (text.includes("on Schematic")) {
                  const refdes = text.replace("on Schematic", "").replace(/[↗︎\s]/g, "").trim();
                  if (refdes) {
                    const resolved = resolveComponentRefdes(circuitJson || [], refdes);
                    if (resolved) {
                      onSelectComponent(resolved);
                      return;
                    }
                  }
                }
                const match = text.match(/Hide\s*"([^"]+)"/);
                if (match && match[1]) {
                  const resolved = resolveComponentRefdes(circuitJson || [], match[1]);
                  if (resolved) {
                    onSelectComponent(resolved);
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

    return () => {
      container.removeEventListener("pointerdown", handlePointerDown, true);
      container.removeEventListener("click", handleClick, true);
      container.removeEventListener("contextmenu", handleContextMenu, true);
      document.removeEventListener("click", handleDocumentClick, true);
      observer.disconnect();
    };
  }, [circuitJson, onSelectComponent]);

  return React.createElement(
    "div",
    {
      ref: containerRef,
      style: { width: "100%", height: "100%", position: "relative", overflow: "hidden" },
    },
    React.createElement(BaseCadViewer, {
      ...props,
      circuitJson: circuitJson || [],
      autoRotateDisabled,
      clickToInteractEnabled,
      onViewSchematicComponent: (event: any) => {
        const refdes = event?.refdes || event?.source_component_id;
        if (refdes && onSelectComponent) {
          onSelectComponent(resolveComponentRefdes(circuitJson || [], refdes));
        }
        if (onViewSchematicComponent) {
          onViewSchematicComponent(event);
        }
      },
      onCameraControllerReady: (controller: any) => {
        if (!controller) {
          if (onCameraControllerReady) onCameraControllerReady(controller);
          return;
        }
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
        if (onCameraControllerReady) {
          onCameraControllerReady(wrappedController);
        }
      },
    })
  );
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

    const handleDocumentClick = (e: MouseEvent) => {
      const target = e.target as HTMLElement | null;
      const btn = target?.closest('button, [role="menuitem"]');
      if (btn) {
        const text = btn.textContent || "";
        if (text.includes("on Schematic")) {
          const refdes = text.replace("on Schematic", "").replace(/[↗︎\s]/g, "").trim();
          if (refdes) {
            const resolved = resolveComponentRefdes(circuitJson || [], refdes);
            if (resolved) onSelectComponent(resolved);
          }
        }
      }
    };
    document.addEventListener("click", handleDocumentClick, true);

    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        for (const node of Array.from(mutation.addedNodes)) {
          if (node instanceof HTMLElement) {
            const menu = node.getAttribute("role") === "menu" ? node : node.querySelector('[role="menu"]');
            if (menu) {
              const buttons = menu.querySelectorAll("button, [role=\"menuitem\"]");
              for (const btn of Array.from(buttons)) {
                const text = btn.textContent || "";
                if (text.includes("on Schematic")) {
                  const refdes = text.replace("on Schematic", "").replace(/[↗︎\s]/g, "").trim();
                  if (refdes) {
                    const resolved = resolveComponentRefdes(circuitJson || [], refdes);
                    if (resolved) {
                      onSelectComponent(resolved);
                      return;
                    }
                  }
                }
              }
            }
          }
        }
      }
    });
    observer.observe(document.body, { childList: true, subtree: true });

    return () => {
      document.removeEventListener("click", handleDocumentClick, true);
      observer.disconnect();
    };
  }, [circuitJson, onSelectComponent]);

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
        const refdes = event?.refdes || event?.source_component_id;
        if (refdes && onSelectComponent) {
          onSelectComponent(resolveComponentRefdes(circuitJson || [], refdes));
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
      onSelectComponent,
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

function ResponsiveSchematicViewer({
  circuitJson,
  onSelectComponent,
}: {
  circuitJson: any;
  onSelectComponent?: (name: string) => void;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({ width: 800, height: 600 });
  const normalizedCircuit = React.useMemo(
    () => normalizeCircuitJsonForSchematic(circuitJson),
    [circuitJson]
  );

  useEffect(() => {
    const parent = containerRef.current?.parentElement || containerRef.current;
    if (!parent) return;
    const updateSize = () => {
      const w = parent.clientWidth || window.innerWidth;
      const h = parent.clientHeight || window.innerHeight - 52;
      if (w > 50 && h > 50) {
        setDimensions({ width: w, height: h });
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

  return React.createElement(
    "div",
    {
      ref: containerRef,
      style: { width: "100%", height: "100%", position: "relative", overflow: "hidden" },
    },
    React.createElement(SchematicViewer, {
      key: `${dimensions.width}x${dimensions.height}`,
      circuitJson: normalizedCircuit || [],
      containerStyle: { width: "100%", height: "100%", backgroundColor: "#000000" },
      colorOverrides: {
        schematic: FUSIBLE_SCHEMATIC_PALETTE,
      },
      onSchematicComponentClicked: (event: any) => {
        const id = event?.schematicComponentId || event?.refdes;
        if (id && onSelectComponent) {
          onSelectComponent(resolveComponentRefdes(normalizedCircuit || [], id));
        }
      },
      onViewPcbComponent: (event: any) => {
        const refdes = event?.refdes;
        if (refdes && onSelectComponent) {
          onSelectComponent(resolveComponentRefdes(normalizedCircuit || [], refdes));
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
    React.createElement(ResponsiveSchematicViewer, {
      circuitJson: circuitJson || [],
      onSelectComponent,
    })
  );
}

export {
  convertCircuitJsonToSchematicSvg,
  convertCircuitJsonToPcbSvg,
  colorMap,
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
