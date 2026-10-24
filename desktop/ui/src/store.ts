import type { Project, Settings } from "./types";

export type Failure = { title: string; log: string };

export type State = {
  view: "projects" | "settings";
  loaded: boolean;
  projects: Project[];
  settings: Settings | null;
  opening: string | null;
  failure: Failure | null;
  loadError: string | null;
};

export const state: State = {
  view: "projects",
  loaded: false,
  projects: [],
  settings: null,
  opening: null,
  failure: null,
  loadError: null,
};

let listener: () => void = () => {};

export function onChange(fn: () => void): void {
  listener = fn;
}

export function update(patch: Partial<State>): void {
  Object.assign(state, patch);
  listener();
}
