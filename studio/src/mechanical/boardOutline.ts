import * as THREE from "three";
import { themeHex } from "../theme";

type Point = [number, number];

interface Element {
  type?: string;
  center?: { x: number; y: number };
  width?: number;
  height?: number;
  outline?: { x: number; y: number }[];
}

function elementsOf(payload: unknown): Element[] {
  if (Array.isArray(payload)) return payload as Element[];
  const inner = (payload as { circuit_json?: unknown } | null)?.circuit_json;
  return Array.isArray(inner) ? (inner as Element[]) : [];
}

/** Returns the outline of the first `pcb_board` of a circuit payload in pcb coordinates, or `[]` when it has none. */
export function outlineOf(payload: unknown): Point[] {
  const board = elementsOf(payload).find((element) => element.type === "pcb_board");
  if (!board) return [];
  if (board.outline && board.outline.length >= 3) return board.outline.map((p) => [p.x, p.y]);
  const { width, height } = board;
  if (!width || !height) return [];
  const { x, y } = board.center ?? { x: 0, y: 0 };
  const [w, h] = [width / 2, height / 2];
  return [
    [x - w, y - h],
    [x + w, y - h],
    [x + w, y + h],
    [x - w, y + h],
  ];
}

/** Fetches the outline of `board`; an unavailable board or circuit yields no outline instead of an error. */
export async function fetchOutline(board: string): Promise<Point[]> {
  if (!board) return [];
  try {
    const response = await fetch(`/api/circuit?board=${encodeURIComponent(board)}`, { cache: "no-store" });
    return response.ok ? outlineOf(await response.json()) : [];
  } catch {
    return [];
  }
}

/** Builds a closed line in the board frame (Z = 0, the board's reference plane). */
export function buildOutlineLine(outline: Point[]): THREE.LineLoop {
  const geometry = new THREE.BufferGeometry().setFromPoints(outline.map(([x, y]) => new THREE.Vector3(x, y, 0)));
  const line = new THREE.LineLoop(geometry, new THREE.LineBasicMaterial({ color: themeHex("green") }));
  line.name = "board-outline";
  return line;
}
