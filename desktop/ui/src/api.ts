import { mockInvoke } from "./mock";
import { subscribeProgress } from "./mockUpdate";
import type {
  CreateArgs, Inspection, Project, Settings, ToolchainStatus, UpdateCheck, UpdateProgress,
} from "./types";

type Invoke = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

type Listener = (event: { payload: UpdateProgress }) => void;

type TauriGlobal = { invoke?: Invoke; transformCallback?: (cb: Listener) => number };

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

export async function onUpdateProgress(cb: (progress: UpdateProgress) => void): Promise<void> {
  const tauri = (window as unknown as { __TAURI_INTERNALS__?: TauriGlobal }).__TAURI_INTERNALS__;
  if (!tauri?.invoke || !tauri.transformCallback) return subscribeProgress(cb);
  const handler = tauri.transformCallback((event) => cb(event.payload));
  await tauri.invoke("plugin:event|listen", { event: "update-progress", target: { kind: "Any" }, handler });
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
  checkUpdate: (auto: boolean) => call<UpdateCheck>("check_update", { auto }),
  installUpdate: () => call<void>("install_update"),
  restartApp: () => call<void>("restart_app"),
  openUrl: (url: string) => call<void>("plugin:opener|open_url", { url }),
  appVersion: () => call<string>("plugin:app|version"),
  pickFolder: (title: string, defaultPath?: string) =>
    call<string | null>("plugin:dialog|open", { options: { directory: true, multiple: false, title, defaultPath } }),
};
