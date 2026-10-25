import * as THREE from "three";
import { animateCameraPreset, type Vec3 } from "../orbit";
import type { Stage } from "./stage";

function contentBounds(stage: Stage): THREE.Sphere {
  const box = new THREE.Box3().setFromObject(stage.content);
  return box.isEmpty() ? new THREE.Sphere(new THREE.Vector3(), 50) : box.getBoundingSphere(new THREE.Sphere());
}

function fitDistance(stage: Stage, radius: number): number {
  const vertical = THREE.MathUtils.degToRad(stage.camera.fov) / 2;
  const horizontal = Math.atan(Math.tan(vertical) * Math.max(stage.camera.aspect, 1e-3));
  return (radius * 1.2) / Math.sin(Math.min(vertical, horizontal));
}

/** Animates the stage camera to a preset direction, framing the content bounds. */
export function framePreset(stage: Stage, direction: Vec3, ms: number): void {
  frameSphere(stage, direction, contentBounds(stage), ms);
}

/** Animates the stage camera to look from `direction` at `sphere`, filling the view with it. */
export function frameSphere(stage: Stage, direction: Vec3, sphere: THREE.Sphere, ms: number): void {
  const center: Vec3 = [sphere.center.x, sphere.center.y, sphere.center.z];
  animateCameraPreset(stage, direction, center, fitDistance(stage, sphere.radius), ms);
}
