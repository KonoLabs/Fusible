import * as THREE from "three";
import { GLTFLoader } from "three-stdlib";

async function readError(response: Response): Promise<string> {
  const fallback = `BODY MODEL UNAVAILABLE (HTTP ${response.status})`;
  try {
    const body: { error?: string } = await response.json();
    return body.error || fallback;
  } catch {
    return fallback;
  }
}

function collectGeometries(root: THREE.Object3D): THREE.BufferGeometry[] {
  const geometries: THREE.BufferGeometry[] = [];
  root.updateMatrixWorld(true);
  root.traverse((node) => {
    if (!(node instanceof THREE.Mesh)) return;
    geometries.push((node.geometry as THREE.BufferGeometry).clone().applyMatrix4(node.matrixWorld));
  });
  return geometries;
}

/**
 * Fetches the GLB of the authored body `name` and returns its meshes untransformed: authored bodies are
 * modeled in the board frame (mm, Z-up, X/Y equal to the pcb coordinates). `nonce` forces a fresh request.
 */
export async function fetchAuthored(name: string, board: string, nonce: number): Promise<THREE.BufferGeometry[]> {
  const query = `board=${encodeURIComponent(board)}&rev=${nonce}`;
  const response = await fetch(`/api/bodies/${encodeURIComponent(name)}.glb?${query}`, { cache: "no-store" });
  if (!response.ok) throw new Error(await readError(response));
  const gltf = await new GLTFLoader().parseAsync(await response.arrayBuffer(), "");
  return collectGeometries(gltf.scene);
}
