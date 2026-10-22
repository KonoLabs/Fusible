import * as THREE from "three";
import { mainCapturedCanvas } from "./capture";
import { animatePose, eyeOf, poseFrom, type Vec3 } from "./orbit";

export type CameraPresetName = "TOP" | "ISO" | "FRONT" | "RESET";

export interface CadCameraController {
  animateTo: (config: { position: Vec3; target?: Vec3; up?: Vec3; durationMs?: number }) => void;
  animateToPreset: (preset: string) => void;
}

/** Eye directions of the camera presets in a Z-up frame (CadViewer and the mechanical tab). */
export const Z_UP_PRESETS: Record<CameraPresetName, Vec3> = {
  TOP: [0, 0, 1],
  ISO: [-1, -1, 1],
  FRONT: [0, -1, 0.1],
  RESET: [-1, -1, 1],
};

const Z_UP = new THREE.Vector3(0, 0, 1);

function estimateTarget(camera: THREE.Camera, focus: THREE.Vector3): THREE.Vector3 {
  const forward = camera.getWorldDirection(new THREE.Vector3());
  const along = Math.max(1e-3, focus.clone().sub(camera.position).dot(forward));
  return camera.position.clone().addScaledVector(forward, along);
}

/** Drives the CadViewer camera to a preset along one continuous spherical curve, through its public animateTo. */
export function animateCadCameraPreset(
  container: HTMLElement,
  controller: CadCameraController,
  preset: CameraPresetName
): void {
  const camera = mainCapturedCanvas(container)?.__threeCamera;
  if (!camera) {
    controller.animateToPreset(preset === "TOP" ? "Top Down" : preset === "FRONT" ? "Front" : "Top Left Corner");
    return;
  }
  const focus = new THREE.Vector3(0, 0, 0);
  const start = estimateTarget(camera, focus);
  const from = poseFrom(camera.position, start, Z_UP);
  const to = poseFrom(new THREE.Vector3(...Z_UP_PRESETS[preset]), focus, Z_UP);
  to.radius = from.radius;
  animatePose(camera, from, to, (pose) => {
    const eye = eyeOf(pose, Z_UP);
    const target: Vec3 = [pose.target.x, pose.target.y, pose.target.z];
    controller.animateTo({ position: [eye.x, eye.y, eye.z], target, up: [0, 0, 1], durationMs: 0 });
  });
}
