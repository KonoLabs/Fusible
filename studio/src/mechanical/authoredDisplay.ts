import type * as THREE from "three";
import { buildOutlineLine } from "./boardOutline";
import { buildPart } from "./display";

export interface AuthoredDisplay {
  root: THREE.Group;
  dispose: () => void;
}

/** Builds a solid group for an authored body, in the base palette; the shared geometry is not disposed by it. */
export function buildAuthoredDisplay(
  geometries: THREE.BufferGeometry[],
  outline: [number, number][]
): AuthoredDisplay {
  const part = buildPart(geometries, "base", "solid");
  part.group.name = "authored-body";
  const line = outline.length >= 3 ? buildOutlineLine(outline) : null;
  if (line) part.group.add(line);
  return {
    root: part.group,
    dispose: () => {
      line?.geometry.dispose();
      (line?.material as THREE.Material | undefined)?.dispose();
      part.group.removeFromParent();
      part.dispose();
    },
  };
}
