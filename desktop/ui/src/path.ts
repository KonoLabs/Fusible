export const FORBIDDEN = /[<>:"/\\|?*\u0000-\u001f]/;

export function separator(path: string): string {
  return path.includes("\\") && !path.includes("/") ? "\\" : "/";
}

export function joinPath(parent: string, name: string): string {
  const sep = separator(parent);
  const trimmed = parent.replace(/[\\/]+$/, "");
  return `${trimmed}${sep}${name}`;
}

export function finalPath(location: string, name: string, createFolder: boolean): string {
  const base = location.trim();
  const clean = name.trim();
  if (!createFolder || !clean) return base;
  if (!base) return clean;
  return joinPath(base, clean);
}

export function baseName(path: string): string {
  const parts = path.split(/[\\/]+/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

export function suggestName(folder: string): string {
  return baseName(folder).replace(FORBIDDEN, "-").replace(/[. ]+$/, "");
}

export function shortenPath(path: string, max = 42): string {
  if (path.length <= max) return path;
  const sep = separator(path);
  const parts = path.split(/[\\/]+/);
  if (parts.length <= 3) return `...${path.slice(path.length - max + 3)}`;
  const tail = parts.slice(-2).join(sep);
  const head = parts[0] === "" ? sep + (parts[1] ?? "") : parts[0];
  const short = `${head}${sep}...${sep}${tail}`;
  return short.length <= max ? short : `...${sep}${tail}`.slice(-max);
}
