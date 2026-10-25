import * as THREE from "three";
import { themeHex } from "../theme";
import type { MechanicalModel, MechanicalPart } from "./store";

export type MechanicalStyle = "solid" | "overlay";

export interface MechanicalDisplay {
  root: THREE.Group;
  parts: Record<MechanicalPart, THREE.Group>;
  dispose: () => void;
}

const PALETTE: Record<MechanicalPart, { color: string; opacity: number }> = {
  base: { color: "part-base", opacity: 0.16 },
  lid: { color: "olive", opacity: 0.1 },
};

const EDGE_COLOR: Record<MechanicalStyle, string> = { solid: "edge", overlay: "green" };

function surfaceMaterial(part: MechanicalPart, style: MechanicalStyle): THREE.Material {
  const { color: colorName, opacity } = PALETTE[part];
  const color = themeHex(colorName);
  const overlay = style === "overlay";
  return new THREE.MeshStandardMaterial({
    color,
    roughness: 0.72,
    metalness: 0.05,
    side: THREE.DoubleSide,
    transparent: overlay,
    opacity: overlay ? opacity : 1,
    depthWrite: !overlay,
  });
}

function edgeMaterial(style: MechanicalStyle): THREE.LineBasicMaterial {
  const overlay = style === "overlay";
  return new THREE.LineBasicMaterial({
    color: themeHex(EDGE_COLOR[style]),
    transparent: overlay,
    opacity: overlay ? 0.55 : 1,
    depthWrite: !overlay,
  });
}

function ignoreRaycast(object: THREE.Object3D): THREE.Object3D {
  object.raycast = () => undefined;
  return object;
}

export function buildPart(geometries: THREE.BufferGeometry[], part: MechanicalPart, style: MechanicalStyle) {
  const group = new THREE.Group();
  group.name = `mechanical-${part}`;
  const surface = surfaceMaterial(part, style);
  const lines = edgeMaterial(style);
  const owned: THREE.BufferGeometry[] = [];
  for (const geometry of geometries) {
    const edges = new THREE.EdgesGeometry(geometry, 30);
    owned.push(edges);
    group.add(ignoreRaycast(new THREE.Mesh(geometry, surface)));
    group.add(ignoreRaycast(new THREE.LineSegments(edges, lines)));
  }
  const dispose = () => {
    owned.forEach((edges) => edges.dispose());
    surface.dispose();
    lines.dispose();
  };
  return { group, dispose };
}

/** Builds a renderable group for the model, with palette materials; shared model geometry is not disposed by it. */
export function buildMechanicalDisplay(model: MechanicalModel, style: MechanicalStyle): MechanicalDisplay {
  const root = new THREE.Group();
  root.name = `mechanical-${style}`;
  const base = buildPart(model.parts.base, "base", style);
  const lid = buildPart(model.parts.lid, "lid", style);
  root.add(base.group, lid.group);
  if (style === "overlay") root.renderOrder = 10;
  return {
    root,
    parts: { base: base.group, lid: lid.group },
    dispose: () => {
      root.removeFromParent();
      base.dispose();
      lid.dispose();
    },
  };
}

/** Applies lid visibility and the exploded lift to a display built from `model`. */
export function poseMechanicalDisplay(
  display: MechanicalDisplay,
  model: MechanicalModel,
  lid: boolean,
  explode: boolean
): void {
  display.parts.lid.visible = lid;
  display.parts.lid.position.z = explode ? Math.max(12, model.meta.size[2]) : 0;
}
