import type * as THREE from "three";
import { parseMeshes, readError } from "./gltf";

/**
 * Fetches the GLB of the authored body `name` and returns its meshes untransformed: authored bodies are
 * modeled in the board frame (mm, Z-up, X/Y equal to the pcb coordinates). `nonce` forces a fresh request.
 */
export async function fetchAuthored(name: string, nonce: number): Promise<THREE.BufferGeometry[]> {
  const response = await fetch(`/api/bodies/${encodeURIComponent(name)}.glb?rev=${nonce}`, { cache: "no-store" });
  if (!response.ok) throw new Error(await readError(response, "BODY MODEL UNAVAILABLE"));
  const meshes = await parseMeshes(await response.arrayBuffer());
  return meshes.map((mesh) => mesh.geometry);
}
