import { api, message } from "./api";
import { baseName } from "./path";
import { state, update } from "./store";

export async function loadAll(): Promise<void> {
  try {
    const [projects, settings] = await Promise.all([api.listProjects(), api.getSettings()]);
    update({ projects, settings, loaded: true, loadError: null });
    void loadToolchain();
  } catch (error) {
    update({ loaded: true, loadError: message(error) });
  }
}

async function loadToolchain(): Promise<void> {
  try {
    update({ toolchain: await api.toolchainStatus() });
  } catch {
    update({ toolchain: { bun: null, node: null, tsci: null } });
  }
}

export async function refreshProjects(): Promise<void> {
  try {
    update({ projects: await api.listProjects() });
  } catch (error) {
    update({ loadError: message(error) });
  }
}

export async function failureLog(error: unknown): Promise<string> {
  const stored = await api.lastError().catch(() => null);
  return stored || message(error);
}

export async function openProject(path: string, name = baseName(path)): Promise<void> {
  if (state.opening) return;
  update({ opening: path, failure: null });
  try {
    await api.openProject(path);
    update({ opening: null });
    await refreshProjects();
  } catch (error) {
    update({ opening: null, failure: { title: `Could not open ${name}`, log: await failureLog(error) } });
  }
}

export async function forgetProject(path: string): Promise<void> {
  try {
    await api.forgetProject(path);
    await refreshProjects();
  } catch (error) {
    update({ failure: { title: "Could not forget the project", log: message(error) } });
  }
}

export async function revealProject(path: string): Promise<void> {
  try {
    await api.revealProject(path);
  } catch (error) {
    update({ failure: { title: "Could not open the folder", log: message(error) } });
  }
}

export async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    return false;
  }
}
