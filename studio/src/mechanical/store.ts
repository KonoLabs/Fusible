import type * as THREE from "three";
import { fetchModel } from "./fetch";

export interface MechanicalMeta {
  revision: string;
  board_z: number;
  board_thickness: number;
  height: number;
  wall: number;
  size: [number, number, number];
}

export type MechanicalPart = "base" | "lid";

export interface MechanicalModel {
  revision: string;
  meta: MechanicalMeta;
  parts: Record<MechanicalPart, THREE.BufferGeometry[]>;
}

export type MechanicalState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "error"; error: string }
  | { status: "ready"; model: MechanicalModel };

type Listener = (state: MechanicalState) => void;

const listeners = new Set<Listener>();
const consumers = new Set<string>();
let revision: string | null = null;
let state: MechanicalState = { status: "idle" };
let requested: string | null = null;
let shown: MechanicalModel | null = null;

function emit(next: MechanicalState): void {
  state = next;
  listeners.forEach((listener) => listener(next));
  if (next.status === "loading") return;
  const model = next.status === "ready" ? next.model : null;
  if (shown && shown !== model) disposeModel(shown);
  shown = model;
}

function fail(rev: string, err: Error): void {
  if (revision !== rev) return;
  requested = null;
  emit({ status: "error", error: err.message || String(err) });
}

function disposeModel(model: MechanicalModel): void {
  Object.values(model.parts).flat().forEach((geometry) => geometry.dispose());
}

function refresh(): void {
  if (!consumers.size || revision === null || requested === revision) return;
  const rev = revision;
  requested = rev;
  emit({ status: "loading" });
  fetchModel(rev).then(
    (model) => (revision === rev ? emit({ status: "ready", model }) : disposeModel(model)),
    (err: Error) => fail(rev, err)
  );
}

/** Records the circuit revision from the studio payload and refetches the model when a consumer needs it. */
export function setMechanicalRevision(next: string): void {
  if (next === revision) return;
  revision = next;
  requested = null;
  refresh();
}

/** Marks a consumer (tab or overlay) as needing the model; fetching only happens while one is active. */
export function demandMechanical(consumer: string, active: boolean): void {
  if (active) consumers.add(consumer);
  else consumers.delete(consumer);
  refresh();
}

/** Subscribes to model state changes and immediately replays the current state. Returns an unsubscribe function. */
export function subscribeMechanical(listener: Listener): () => void {
  listeners.add(listener);
  listener(state);
  return () => listeners.delete(listener);
}
