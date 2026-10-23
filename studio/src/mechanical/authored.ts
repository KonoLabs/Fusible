import type * as THREE from "three";
import { fetchAuthored } from "./authoredFetch";
import { errorMessage } from "./gltf";

export interface AuthoredModel {
  key: string;
  name: string;
  geometries: THREE.BufferGeometry[];
}

export type AuthoredState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "error"; error: string }
  | { status: "ready"; model: AuthoredModel };

type Listener = (state: AuthoredState) => void;

const listeners = new Set<Listener>();
const consumers = new Set<string>();
let selected: string | null = null;
let nonce = 0;
let requested: string | null = null;
let state: AuthoredState = { status: "idle" };
let shown: AuthoredModel | null = null;

function emit(next: AuthoredState): void {
  state = next;
  listeners.forEach((listener) => listener(next));
  if (next.status === "loading") return;
  const model = next.status === "ready" ? next.model : null;
  if (shown && shown !== model) shown.geometries.forEach((geometry) => geometry.dispose());
  shown = model;
}

function refresh(): void {
  if (selected === null || !consumers.size) return;
  const name = selected;
  const key = `${name}|${nonce}`;
  if (requested === key) return;
  requested = key;
  emit({ status: "loading" });
  fetchAuthored(name, nonce).then(
    (geometries) => {
      if (requested === key) emit({ status: "ready", model: { key, name, geometries } });
      else geometries.forEach((geometry) => geometry.dispose());
    },
    (reason: unknown) => {
      if (requested === key) emit({ status: "error", error: errorMessage(reason) });
    }
  );
}

/** Selects the authored body shown by the bodies tab; any other selection (or `null`) clears it. */
export function setMechanicalBody(name: string | null, kind: string): void {
  selected = name !== null && kind === "authored" ? name : null;
  nonce++;
  requested = null;
  if (selected === null) emit({ status: "idle" });
  refresh();
}

/** Reports whether an authored body, not the generated enclosure, is selected. */
export function isAuthoredSelected(): boolean {
  return selected !== null;
}

/** Forces the next fetch of the selected authored body, so a changed source file is picked up. */
export function refreshAuthored(): void {
  if (selected === null) return;
  nonce++;
  requested = null;
  refresh();
}

/** Marks a consumer as needing the authored body; fetching only happens while one is active. */
export function demandAuthored(consumer: string, active: boolean): void {
  if (active) consumers.add(consumer);
  else consumers.delete(consumer);
  refresh();
}

/** Subscribes to authored body state changes and replays the current state. Returns an unsubscribe function. */
export function subscribeAuthored(listener: Listener): () => void {
  listeners.add(listener);
  listener(state);
  return () => listeners.delete(listener);
}
