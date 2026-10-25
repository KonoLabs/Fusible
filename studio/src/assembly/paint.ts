import * as THREE from "three";
import { themeHex } from "../theme";
import type { Preview } from "./model";

/** Repaints every collision solid of `preview` with the current theme alert colour. */
export function recolourCollisions(preview: Preview): void {
  const color = themeHex("alert");
  preview.collisions.forEach((node) =>
    node.traverse((child) => {
      if (!(child instanceof THREE.Mesh)) return;
      [child.material].flat().forEach((material) => {
        if (material instanceof THREE.MeshBasicMaterial && material.color.getHex() !== color) {
          material.color.setHex(color);
        }
      });
    }),
  );
}

function disposeMaterial(material: THREE.Material): void {
  Object.values(material).forEach((value) => {
    if (value instanceof THREE.Texture) value.dispose();
  });
  material.dispose();
}

/** Releases the geometry, materials and textures of a preview. */
export function disposePreview(preview: Preview): void {
  preview.root.traverse((node) => {
    if (!(node instanceof THREE.Mesh)) return;
    node.geometry.dispose();
    [node.material].flat().forEach(disposeMaterial);
  });
}
