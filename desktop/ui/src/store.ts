import { loadSelected } from "./selection";
import type { Project, Settings, ToolchainStatus } from "./types";
import { IDLE, type UpdateState } from "./updateState";

export type Failure = { title: string; log: string };

export type State = {
  view: "projects" | "settings";
  loaded: boolean;
  projects: Project[];
  settings: Settings | null;
  toolchain: ToolchainStatus | null;
  update: UpdateState;
  opening: string | null;
  failure: Failure | null;
  loadError: string | null;
  selected: string | null;
  query: string;
};

export const state: State = {
  view: "projects",
  loaded: false,
  projects: [],
  settings: null,
  toolchain: null,
  update: IDLE,
  opening: null,
  failure: null,
  loadError: null,
  selected: loadSelected(),
  query: "",
};

let listener: () => void = () => {};

export function onChange(fn: () => void): void {
  listener = fn;
}

export function update(patch: Partial<State>): void {
  Object.assign(state, patch);
  listener();
}
