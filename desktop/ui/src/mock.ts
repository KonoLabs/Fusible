import { FAILURE_LOG, defaultSettings, manyProjects, toolchain } from "./fixtures";
import { checkUpdate, installUpdate } from "./mockUpdate";
import type { CreateArgs, Project, Settings } from "./types";

type Args = Record<string, unknown> | undefined;

const query = new URLSearchParams(location.search);
const scenario = query.get("scenario") ?? "many";
const store = { projects: scenario === "empty" ? [] : manyProjects(), settings: defaultSettings(), error: "" };

function delay<T>(value: T, ms = 60): Promise<T> {
  return new Promise((done) => setTimeout(() => done(value), ms));
}

function openProject(args: Args): Promise<void> {
  if (scenario === "opening") return new Promise<void>(() => {});
  if (scenario === "failure") {
    store.error = FAILURE_LOG;
    return Promise.reject(new Error(FAILURE_LOG.split("\n")[0]));
  }
  const target = store.projects.find((p) => p.path === args?.path);
  if (target) target.last_opened = new Date().toISOString();
  return delay(undefined, 900);
}

function createProject(args: Args): Promise<Project> {
  const a = args as unknown as CreateArgs;
  const path = a.create_folder ? `${a.parent}/${a.name}` : a.parent;
  const created: Project = {
    path, name: a.name, boards: 0, firmware: 0, bodies: 0, last_opened: null, missing: false,
  };
  store.projects.unshift(created);
  return delay(created, 300);
}

function setSettings(args: Args): Promise<Settings> {
  store.settings = args?.settings as Settings;
  return delay(store.settings, 30);
}

function forget(args: Args): Promise<void> {
  store.projects = store.projects.filter((p) => p.path !== args?.path);
  return delay(undefined);
}

function openUrl(args: Args): Promise<void> {
  window.open(String(args?.url), "_blank");
  return delay(undefined);
}

function touchChecked(): void {
  store.settings.last_update_check = Math.floor(Date.now() / 1000);
}

const handlers: Record<string, (args: Args) => Promise<unknown>> = {
  list_projects: () => delay([...store.projects]),
  open_project: openProject,
  create_project: createProject,
  inspect_folder: (a) => delay({ has_manifest: String(a?.path).endsWith("FieldCam"), name: null }),
  forget_project: forget,
  reveal_project: () => delay(undefined),
  get_settings: () => delay({ ...store.settings }),
  set_settings: setSettings,
  toolchain_status: () => delay(toolchain()),
  last_error: () => delay(store.error || null),
  open_projects: () => delay(undefined),
  check_update: (a) => checkUpdate(Boolean(a?.auto), touchChecked),
  install_update: installUpdate,
  restart_app: () => delay(undefined),
  "plugin:opener|open_url": openUrl,
  "plugin:app|version": () => delay("0.14.3"),
  "plugin:dialog|open": () => delay(query.get("pick") ?? "/home/yann/Code/Scratch"),
};

export function mockInvoke(cmd: string, args?: Args): Promise<unknown> {
  const handler = handlers[cmd];
  return handler ? handler(args) : Promise.reject(new Error(`Unknown command ${cmd}`));
}
