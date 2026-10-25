import type { Project } from "./types";

const KEY = "fusible.selected";

export function sortRecent(projects: Project[]): Project[] {
  const stamp = (p: Project) => (p.last_opened ? Date.parse(p.last_opened) || 0 : 0);
  return [...projects].sort((a, b) => stamp(b) - stamp(a));
}

export function filterProjects(projects: Project[], query: string): Project[] {
  const needle = query.trim().toLowerCase();
  const list = sortRecent(projects);
  if (!needle) return list;
  return list.filter((p) => p.name.toLowerCase().includes(needle) || p.path.toLowerCase().includes(needle));
}

export function pickSelection(visible: Project[], selected: string | null): string | null {
  if (selected && visible.some((p) => p.path === selected)) return selected;
  return visible[0]?.path ?? null;
}

export function moveSelection(visible: Project[], selected: string | null, step: number): string | null {
  if (!visible.length) return null;
  const at = visible.findIndex((p) => p.path === selected);
  if (at < 0) return visible[step < 0 ? visible.length - 1 : 0].path;
  return visible[Math.min(visible.length - 1, Math.max(0, at + step))].path;
}

export function loadSelected(): string | null {
  try {
    return localStorage.getItem(KEY);
  } catch {
    return null;
  }
}

export function saveSelected(path: string | null): void {
  try {
    if (path) localStorage.setItem(KEY, path);
    else localStorage.removeItem(KEY);
  } catch {
    return;
  }
}
