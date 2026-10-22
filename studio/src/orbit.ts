import * as THREE from "three";

export type Vec3 = [number, number, number];

export interface OrbitPose {
  target: THREE.Vector3;
  radius: number;
  polar: number;
  azimuth: number;
}

export interface OrbitRig {
  camera: THREE.Camera;
  controls: { target: THREE.Vector3; update: () => void };
}

const POLAR_EPSILON = 1e-3;
const running = new WeakMap<object, number>();

function upSpace(up: THREE.Vector3): THREE.Quaternion {
  return new THREE.Quaternion().setFromUnitVectors(up.clone().normalize(), new THREE.Vector3(0, 1, 0));
}

/** Expresses an eye position around `target` as a spherical pose in the frame where `up` is the pole. */
export function poseFrom(eye: THREE.Vector3, target: THREE.Vector3, up: THREE.Vector3): OrbitPose {
  const offset = eye.clone().sub(target).applyQuaternion(upSpace(up));
  const { radius, phi, theta } = new THREE.Spherical().setFromVector3(offset);
  const polar = THREE.MathUtils.clamp(phi, POLAR_EPSILON, Math.PI - POLAR_EPSILON);
  return { target: target.clone(), radius, polar, azimuth: theta };
}

/** Returns the eye position of a spherical pose. */
export function eyeOf(pose: OrbitPose, up: THREE.Vector3): THREE.Vector3 {
  const offset = new THREE.Vector3().setFromSphericalCoords(pose.radius, pose.polar, pose.azimuth);
  return offset.applyQuaternion(upSpace(up).invert()).add(pose.target);
}

/** Blends two poses at linear progress `t` with an ease-in-out cubic, turning the azimuth the short way round. */
export function blendPose(from: OrbitPose, to: OrbitPose, t: number): OrbitPose {
  const k = t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
  const turn = THREE.MathUtils.euclideanModulo(to.azimuth - from.azimuth + Math.PI, Math.PI * 2) - Math.PI;
  return {
    target: from.target.clone().lerp(to.target, k),
    radius: THREE.MathUtils.lerp(from.radius, to.radius, k),
    polar: THREE.MathUtils.lerp(from.polar, to.polar, k),
    azimuth: from.azimuth + turn * k,
  };
}

/** Animates along one eased spherical curve, calling `apply` each frame. A new call on `key` cancels the old one. */
export function animatePose(key: object, from: OrbitPose, to: OrbitPose, apply: (pose: OrbitPose) => void, ms = 600) {
  cancelAnimationFrame(running.get(key) ?? 0);
  const start = performance.now();
  const step = () => {
    const t = Math.min(1, (performance.now() - start) / ms);
    apply(blendPose(from, to, t));
    if (t < 1) running.set(key, requestAnimationFrame(step));
    else running.delete(key);
  };
  running.set(key, requestAnimationFrame(step));
}

/** Moves an OrbitControls rig to look at `target` from `direction` at `radius`, never touching `camera.up`. */
export function animateCameraPreset(rig: OrbitRig, direction: Vec3, target: Vec3, radius: number, ms = 600): void {
  const up = rig.camera.up;
  const focus = new THREE.Vector3(...target);
  const goal = poseFrom(new THREE.Vector3(...direction).add(focus), focus, up);
  goal.radius = radius;
  const from = poseFrom(rig.camera.position, rig.controls.target, up);
  animatePose(rig.camera, from, goal, (pose) => {
    rig.camera.position.copy(eyeOf(pose, up));
    rig.controls.target.copy(pose.target);
    rig.controls.update();
  }, ms);
}
