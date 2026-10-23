import type * as THREE from "three";
import { buildPart } from "./display";

export interface AuthoredDisplay {
  root: THREE.Group;
  dispose: () => void;
}

/** Builds a solid group for an authored body, in the base palette; the shared geometry is not disposed by it. */
export function buildAuthoredDisplay(geometries: THREE.BufferGeometry[]): AuthoredDisplay {
  const part = buildPart(geometries, "base", "solid");
  part.group.name = "authored-body";
  return {
    root: part.group,
    dispose: () => {
      part.group.removeFromParent();
      part.dispose();
    },
  };
}
