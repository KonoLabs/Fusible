import type { Tool, ToolchainStatus } from "./types";

function line(name: string, tool: Tool | null | undefined): string {
  return tool ? `${name}: ${tool.version} (${tool.path})` : `${name}: not found`;
}

export function toolchainReport(status: ToolchainStatus, appVersion: string): string {
  return [
    `Fusible Studio ${appVersion}`,
    `fusible ${status.fusible_version ?? "unknown"}`,
    line("bun", status.bun),
    line("node", status.node),
    line("tsci", status.tsci),
    `bundled bun: ${status.bundled_bun ?? "none"}`,
  ].join("\n");
}
