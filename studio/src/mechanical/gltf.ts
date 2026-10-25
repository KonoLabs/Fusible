import * as THREE from "three";
import { GLTFLoader } from "three-stdlib";

/** Returns a readable message for a rejection reason of any type. */
export function errorMessage(reason: unknown): string {
  return reason instanceof Error && reason.message ? reason.message : String(reason);
}

/** Reads the `error` text of a failed response, or `fallback` when the body is not JSON with a string error. */
export async function readError(response: Response, fallback: string): Promise<string> {
  const text = `${fallback} (HTTP ${response.status})`;
  try {
    const body: unknown = await response.json();
    const error = typeof body === "object" && body !== null ? (body as { error?: unknown }).error : undefined;
    return typeof error === "string" && error ? error : text;
  } catch {
    return text;
  }
}

/** Parses a GLB into its scene graph, keeping node names and materials. */
export async function parseScene(buffer: ArrayBuffer): Promise<THREE.Group> {
  const gltf = await new GLTFLoader().parseAsync(buffer, "");
  return gltf.scene;
}

/** Parses a GLB and returns every mesh with its world transform baked into a cloned geometry. */
export async function parseMeshes(
  buffer: ArrayBuffer
): Promise<{ node: THREE.Mesh; geometry: THREE.BufferGeometry }[]> {
  const scene = await parseScene(buffer);
  const meshes: { node: THREE.Mesh; geometry: THREE.BufferGeometry }[] = [];
  scene.updateMatrixWorld(true);
  scene.traverse((node) => {
    if (!(node instanceof THREE.Mesh)) return;
    const geometry = (node.geometry as THREE.BufferGeometry).clone().applyMatrix4(node.matrixWorld);
    meshes.push({ node, geometry });
  });
  return meshes;
}
