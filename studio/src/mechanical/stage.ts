import * as THREE from "three";
import { OrbitControls } from "three-stdlib";
import type { Vec3 } from "../orbit";
import { themeHex } from "../theme";

export interface Stage {
  host: HTMLElement;
  renderer: THREE.WebGLRenderer;
  scene: THREE.Scene;
  camera: THREE.PerspectiveCamera;
  controls: OrbitControls;
  content: THREE.Group;
  grid: THREE.GridHelper;
}

function directional(color: number, intensity: number, position: Vec3): THREE.DirectionalLight {
  const light = new THREE.DirectionalLight(color, intensity);
  light.position.set(...position);
  return light;
}

function addLights(scene: THREE.Scene, camera: THREE.Camera): void {
  const hemi = new THREE.HemisphereLight(themeHex("light"), themeHex("stage-ground"), 0.6);
  hemi.position.set(0, 0, 1);
  scene.add(new THREE.AmbientLight(themeHex("light"), 0.25), hemi);
  scene.add(directional(themeHex("light"), 1.25, [70, -80, 130]));
  scene.add(directional(themeHex("light-fill"), 0.5, [-80, 70, 50]));
  scene.add(directional(themeHex("light"), 0.85, [40, -50, -110]));
  camera.add(directional(themeHex("light"), 0.7, [0, 0, 1]));
  scene.add(camera);
}

function createGrid(): THREE.GridHelper {
  const grid = new THREE.GridHelper(400, 80, themeHex("grid-major"), themeHex("edge"));
  grid.rotation.x = Math.PI / 2;
  grid.position.z = -0.05;
  return grid;
}

/** Creates the Z-up three.js stage of the mechanical tab inside `host`, styled like the native 3D view. */
export function createStage(host: HTMLElement): Stage {
  const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
  renderer.setPixelRatio(window.devicePixelRatio || 1);
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.05;
  renderer.domElement.style.display = "block";
  host.appendChild(renderer.domElement);
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(themeHex("bg"));
  const camera = new THREE.PerspectiveCamera(45, 1, 0.1, 5000);
  camera.up.set(0, 0, 1);
  camera.position.set(-120, -120, 120);
  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.dampingFactor = 0.05;
  addLights(scene, camera);
  const grid = createGrid();
  const content = new THREE.Group();
  scene.add(grid, content);
  return { host, renderer, scene, camera, controls, content, grid };
}

/** Matches the renderer and camera to the host size; returns false while the host is hidden. */
export function resizeStage(stage: Stage): boolean {
  const width = stage.host.clientWidth;
  const height = stage.host.clientHeight;
  if (!width || !height) return false;
  const canvas = stage.renderer.domElement;
  if (canvas.width !== Math.round(width * stage.renderer.getPixelRatio()) || stage.camera.aspect !== width / height) {
    stage.camera.aspect = width / height;
    stage.camera.updateProjectionMatrix();
    stage.renderer.setSize(width, height);
  }
  return true;
}
