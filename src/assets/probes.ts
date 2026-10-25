import type { OcctKernel, ShapeHandle } from "occt-wasm";

export type Vec2 = [number, number];
export type Vec3 = [number, number, number];
export type Shape =
  | { kind: "prism"; points: Vec2[]; z0: number; z1: number }
  | { kind: "box"; min: Vec3; max: Vec3 }
  | { kind: "cylinder"; center: Vec2; r: number; z0: number; z1: number; clip?: Vec2[] }
  | { kind: "ring"; center: Vec2; r0: number; r1: number; z0: number; z1: number; quadrant: number };
export type Probe = Shape & { id: string; collide: boolean; show: boolean; matrix: number[] };

const QUADRANTS: Vec2[] = [
  [1, 1],
  [-1, 1],
  [-1, -1],
  [1, -1],
];

const prism = (k: OcctKernel, points: Vec2[], z0: number, z1: number) => {
  const ring = points.map(([x, y]) => ({ x, y, z: z0 }));
  const wire = k.makeWire(ring.map((p, i) => k.makeLineEdge(p, ring[(i + 1) % ring.length])));
  return k.extrude(k.makeFace(wire), 0, 0, z1 - z0);
};

const cylinder = (k: OcctKernel, [x, y]: Vec2, r: number, z0: number, z1: number) =>
  k.translate(k.makeCylinder(r, z1 - z0), x, y, z0);

export const buildProbe = (k: OcctKernel, p: Probe): ShapeHandle => {
  switch (p.kind) {
    case "prism":
      return prism(k, p.points, p.z0, p.z1);
    case "box":
      return k.makeBoxFromCorners(
        { x: p.min[0], y: p.min[1], z: p.min[2] },
        { x: p.max[0], y: p.max[1], z: p.max[2] },
      );
    case "cylinder": {
      const round = cylinder(k, p.center, p.r, p.z0, p.z1);
      return p.clip ? k.common(round, prism(k, p.clip, p.z0 - 1, p.z1 + 1)) : round;
    }
    case "ring": {
      const outer = cylinder(k, p.center, p.r1, p.z0, p.z1);
      const ring = k.cut(outer, cylinder(k, p.center, p.r0, p.z0, p.z1));
      const [sx, sy] = QUADRANTS[p.quadrant];
      const reach = p.r1 + 1;
      const [cx, cy] = p.center;
      const quadrant = k.makeBoxFromCorners(
        { x: cx, y: cy, z: p.z0 - 1 },
        { x: cx + sx * reach, y: cy + sy * reach, z: p.z1 + 1 },
      );
      return k.common(ring, quadrant);
    }
  }
};
