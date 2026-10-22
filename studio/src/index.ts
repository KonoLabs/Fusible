import React, { useState, useEffect, useRef } from "react";
import { rootSlot, renderIntoSlot } from "./roots";
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
import { Mechanical } from "./mechanical";
import { animateCameraPreset } from "./orbit";
import { animateCadCameraPreset, type CadCameraController } from "./cadCamera";
import { installRenderCapture, mainCapturedCanvas } from "./capture";
import { useContextMenuIntercept } from "./contextMenu";
import { renderPcbViewer } from "./pcbViewer";
import { renderSchematicViewer } from "./schematicViewer";
import { determineRefdesFrom3dObject, resolveComponentRefdes } from "./refdes";
import { FUSIBLE_SCHEMATIC_PALETTE, normalizeCircuitJsonForSchematic, renderFusibleSchematicSvg } from "./schematic";

addCollection(pixelarticons as any);

if (typeof window !== "undefined") {
  (window as any).THREE = Object.assign({}, THREE, { OrbitControls });
}

if (typeof window !== "undefined") installRenderCapture();

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
  useContextMenuIntercept(circuitJson || [], onSelectComponent);

  useEffect(() => {
    const container = containerRef.current;
    if (!container || !onSelectComponent) return;

    const handlePointerDown = (e: PointerEvent) => {
      pointerStartRef.current = { x: e.clientX, y: e.clientY };
    };

    const handleRaycast = (clientX: number, clientY: number) => {
      const canvas = mainCapturedCanvas(container);
      if (!canvas) return;
      const camera = canvas.__threeCamera;
      const scene = canvas.__threeScene;
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

      const meshes = hits.filter((hit) => hit.object instanceof THREE.Mesh);
      const refdes = meshes.map((hit) => determineRefdesFrom3dObject(circuitJson || [], hit.object)).find(Boolean);
      if (refdes) onSelectComponent(refdes);
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

    return () => {
      container.removeEventListener("pointerdown", handlePointerDown, true);
      container.removeEventListener("click", handleClick, true);
      container.removeEventListener("contextmenu", handleContextMenu, true);
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
      onCameraControllerReady: (controller: CadCameraController | null) => {
        window.cadCameraController = controller ?? undefined;
        if (onCameraControllerReady) onCameraControllerReady(controller);
      },
    })
  );
}

const cadSlot = rootSlot();

export function render3dViewer(
  container: HTMLElement,
  circuitJson: any,
  onSelectComponent?: (name: string) => void
) {
  const props = { circuitJson: circuitJson || [], onSelectComponent };
  renderIntoSlot(cadSlot, container, React.createElement(CadViewer, props));
}

export {
  FUSIBLE_SCHEMATIC_PALETTE,
  normalizeCircuitJsonForSchematic,
  renderFusibleSchematicSvg,
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
  animateCameraPreset,
  animateCadCameraPreset,
  ...Mechanical,
};

declare global {
  interface Window {
    Tscircuit: typeof Tscircuit;
    cadCameraController?: CadCameraController;
  }
}

if (typeof window !== "undefined") {
  window.Tscircuit = Tscircuit;
  window.dispatchEvent(new CustomEvent("tscircuit-ready"));
}

export default Tscircuit;
