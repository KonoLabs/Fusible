import { mockInvoke } from "./mock";
import type { CreateArgs, Inspection, Project, Settings, ToolchainStatus } from "./types";

type Invoke = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

type TauriGlobal = { invoke?: Invoke };

export function isMock(): boolean {
  const tauri = (window as unknown as { __TAURI_INTERNALS__?: TauriGlobal }).__TAURI_INTERNALS__;
  return !tauri?.invoke;
}

function resolve(): Invoke {
  const tauri = (window as unknown as { __TAURI_INTERNALS__?: TauriGlobal }).__TAURI_INTERNALS__;
  return tauri?.invoke ?? mockInvoke;
}

export function message(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return String(error);
}

function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return resolve()(cmd, args) as Promise<T>;
}

export const api = {
  listProjects: () => call<Project[]>("list_projects"),
  openProject: (path: string) => call<void>("open_project", { path }),
  createProject: (args: CreateArgs) => call<Project>("create_project", { ...args }),
  inspectFolder: (path: string) => call<Inspection>("inspect_folder", { path }),
  forgetProject: (path: string) => call<void>("forget_project", { path }),
  revealProject: (path: string) => call<void>("reveal_project", { path }),
  getSettings: () => call<Settings>("get_settings"),
  setSettings: (settings: Settings) => call<Settings>("set_settings", { settings }),
  toolchainStatus: () => call<ToolchainStatus>("toolchain_status"),
  lastError: () => call<string | null>("last_error"),
  appVersion: () => call<string>("plugin:app|version"),
  pickFolder: (title: string, defaultPath?: string) =>
    call<string | null>("plugin:dialog|open", { options: { directory: true, multiple: false, title, defaultPath } }),
};
