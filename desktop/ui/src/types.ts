export type Project = {
  path: string;
  name: string;
  boards: number;
  firmware: number;
  bodies: number;
  last_opened: string | null;
  missing: boolean;
};

export type Settings = {
  projects_dir: string;
  default_currency: string;
  port: number;
  open_last_project: boolean;
  theme: "system" | "dark" | "light";
  auto_update: boolean;
  last_update_check: number | null;
};

export type Inspection = { has_manifest: boolean; name: string | null };

export type Tool = { path: string; version: string };

export type ToolchainStatus = {
  bun: Tool | null;
  node: Tool | null;
  tsci: Tool | null;
  bundled_bun?: string | null;
  fusible_version?: string | null;
};

export type CreateArgs = {
  parent: string;
  name: string;
  create_folder: boolean;
  currency: string;
};

export type Theme = Settings["theme"];

export type UpdateCheck =
  | { status: "skipped" }
  | { status: "up_to_date" }
  | { status: "available"; version: string; notes: string; date: string | null }
  | { status: "unsupported"; url: string };

export type UpdateProgress = { downloaded: number; total: number | null };
