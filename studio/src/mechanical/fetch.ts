import * as THREE from "three";
import { GLTFLoader } from "three-stdlib";
import type { MechanicalMeta, MechanicalModel, MechanicalPart } from "./store";

async function readError(response: Response): Promise<string> {
  try {
    const body: { error?: string } = await response.json();
    return body.error || `MECHANICAL MODEL UNAVAILABLE (HTTP ${response.status})`;
  } catch {
    return `MECHANICAL MODEL UNAVAILABLE (HTTP ${response.status})`;
  }
}

function collectParts(root: THREE.Object3D): Record<MechanicalPart, THREE.BufferGeometry[]> {
  const parts: Record<MechanicalPart, THREE.BufferGeometry[]> = { base: [], lid: [] };
  root.updateMatrixWorld(true);
  root.traverse((node) => {
    if (!(node instanceof THREE.Mesh)) return;
    let owner: THREE.Object3D | null = node;
    while (owner && owner.name !== "lid" && owner.name !== "base") owner = owner.parent;
    const geometry = (node.geometry as THREE.BufferGeometry).clone().applyMatrix4(node.matrixWorld);
    parts[owner?.name === "lid" ? "lid" : "base"].push(geometry);
  });
  return parts;
}

/** Fetches the metadata then the GLB of revision `rev` of `board` and splits the meshes into base and lid geometry. */
export async function fetchModel(rev: string, board: string): Promise<MechanicalModel> {
  const query = `rev=${encodeURIComponent(rev)}&board=${encodeURIComponent(board)}`;
  const metaResponse = await fetch(`/api/mechanical?${query}`);
  if (!metaResponse.ok) throw new Error(await readError(metaResponse));
  const meta: MechanicalMeta = await metaResponse.json();
  const glbResponse = await fetch(`/api/mechanical.glb?${query}`);
  if (!glbResponse.ok) throw new Error(await readError(glbResponse));
  const gltf = await new GLTFLoader().parseAsync(await glbResponse.arrayBuffer(), "");
  return { revision: rev, meta, parts: collectParts(gltf.scene) };
}
