import * as THREE from "three";
import { parseScene, readError } from "../mechanical/gltf";
import { themeHex } from "../theme";

export type Point = [number, number, number];

export interface Violation {
  rule: string;
  message: string;
  parts: string[];
  volume?: number;
  at?: Point;
  collision?: string;
}

export interface Report {
  version: number;
  fuse: "intact" | "blown";
  violations: Violation[];
}

export interface Overview {
  declared: boolean;
  report: Report | null;
  report_error?: string | null;
  model: string | null;
}

export interface Preview {
  url: string;
  root: THREE.Group;
  collisions: Map<string, THREE.Object3D>;
}

const COLLISION_PREFIX = "collision:";
const COLLISION_OPACITY = 0.6;

/** Fetches the assembly report and the address of its preview model. */
export async function fetchOverview(): Promise<Overview> {
  const response = await fetch("/api/assembly");
  if (!response.ok) throw new Error(await readError(response, "ASSEMBLY UNAVAILABLE"));
  return response.json();
}

function collisionMaterial(): THREE.MeshBasicMaterial {
  return new THREE.MeshBasicMaterial({
    color: themeHex("alert"),
    transparent: true,
    opacity: COLLISION_OPACITY,
    depthTest: false,
    depthWrite: false,
    side: THREE.DoubleSide,
  });
}

function paintRed(node: THREE.Object3D): void {
  node.traverse((child) => {
    if (!(child instanceof THREE.Mesh)) return;
    child.material = collisionMaterial();
    child.renderOrder = 10;
  });
}

/** Paints every `collision:*` node red on top of the scene and returns them by name. */
export function paintCollisions(root: THREE.Object3D): Map<string, THREE.Object3D> {
  const found = new Map<string, THREE.Object3D>();
  root.traverse((node) => {
    if (!node.name.startsWith(COLLISION_PREFIX) || found.has(node.name)) return;
    found.set(node.name, node);
    paintRed(node);
  });
  return found;
}

/** Downloads the preview GLB at `url` and paints its collisions. */
export async function loadPreview(url: string): Promise<Preview> {
  const response = await fetch(url);
  if (!response.ok) throw new Error(await readError(response, "ASSEMBLY PREVIEW UNAVAILABLE"));
  const root = await parseScene(await response.arrayBuffer());
  return { url, root, collisions: paintCollisions(root) };
}

/** The sphere to frame for a violation: its collision solid, else a ball around its `at` point. */
export function violationSphere(violation: Violation, preview: Preview): THREE.Sphere | null {
  const node = violation.collision ? preview.collisions.get(violation.collision) : undefined;
  const box = node ? new THREE.Box3().setFromObject(node) : new THREE.Box3();
  if (!box.isEmpty()) return box.getBoundingSphere(new THREE.Sphere());
  if (!violation.at) return null;
  return new THREE.Sphere(new THREE.Vector3(...violation.at), 0);
}
