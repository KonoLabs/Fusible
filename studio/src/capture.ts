import * as THREE from "three";

export interface CapturedCanvas extends HTMLCanvasElement {
  __threeScene?: THREE.Scene;
  __threeCamera?: THREE.Camera;
  __threeRenderer?: THREE.WebGLRenderer;
}

interface CaptureWindow {
  __lastThreeScene?: THREE.Object3D;
  __lastThreeCamera?: THREE.Camera;
}

type RenderFn = (scene: THREE.Object3D, camera: THREE.Camera) => void;

function wrapRender(renderer: THREE.WebGLRenderer, render: RenderFn): RenderFn {
  return function (scene, camera) {
    const canvas = renderer.domElement as CapturedCanvas;
    canvas.__threeScene = scene as THREE.Scene;
    canvas.__threeCamera = camera;
    canvas.__threeRenderer = renderer;
    Object.assign(window as CaptureWindow, { __lastThreeScene: scene, __lastThreeCamera: camera });
    render.call(renderer, scene, camera);
  };
}

/**
 * Tags every WebGLRenderer canvas with the scene and camera it last rendered. three r165 assigns `render` per
 * instance in the constructor, so a prototype accessor intercepts that assignment and wraps the method.
 */
export function installRenderCapture(): void {
  const proto = THREE.WebGLRenderer.prototype as THREE.WebGLRenderer & { __fusiblePatched?: boolean };
  if (proto.__fusiblePatched) return;
  Object.defineProperty(proto, "render", {
    configurable: true,
    set(this: THREE.WebGLRenderer, render: RenderFn) {
      Object.defineProperty(this, "render", { value: wrapRender(this, render), writable: true, configurable: true });
    },
  });
  proto.__fusiblePatched = true;
}

/** Returns the largest canvas in `container` that the render capture tagged, skipping gizmo canvases. */
export function mainCapturedCanvas(container: HTMLElement | null): CapturedCanvas | null {
  if (!container) return null;
  const canvases = Array.from(container.querySelectorAll("canvas")) as CapturedCanvas[];
  const tagged = canvases.filter((canvas) => canvas.__threeScene && canvas.clientWidth * canvas.clientHeight > 0);
  tagged.sort((a, b) => b.clientWidth * b.clientHeight - a.clientWidth * a.clientHeight);
  return tagged[0] ?? null;
}
