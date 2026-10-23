import type * as THREE from "three";
import { parseMeshes, readError } from "./gltf";
import type { MechanicalMeta, MechanicalModel, MechanicalPart } from "./store";

function owningPart(node: THREE.Object3D): MechanicalPart {
  let owner: THREE.Object3D | null = node;
  while (owner && owner.name !== "lid" && owner.name !== "base") owner = owner.parent;
  return owner?.name === "lid" ? "lid" : "base";
}

/** Fetches the metadata then the GLB of revision `rev` of `board` and splits the meshes into base and lid geometry. */
export async function fetchModel(rev: string, board: string): Promise<MechanicalModel> {
  const query = `rev=${encodeURIComponent(rev)}&board=${encodeURIComponent(board)}`;
  const failure = "MECHANICAL MODEL UNAVAILABLE";
  const metaResponse = await fetch(`/api/mechanical?${query}`);
  if (!metaResponse.ok) throw new Error(await readError(metaResponse, failure));
  const meta: MechanicalMeta = await metaResponse.json();
  const glbResponse = await fetch(`/api/mechanical.glb?${query}`);
  if (!glbResponse.ok) throw new Error(await readError(glbResponse, failure));
  const parts: Record<MechanicalPart, THREE.BufferGeometry[]> = { base: [], lid: [] };
  for (const { node, geometry } of await parseMeshes(await glbResponse.arrayBuffer())) {
    parts[owningPart(node)].push(geometry);
  }
  return { revision: rev, meta, parts };
}
