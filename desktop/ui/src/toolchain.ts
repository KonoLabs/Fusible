import { api, message } from "./api";
import { copyButton } from "./copyButton";
import { h, icon } from "./dom";
import { toolchainReport } from "./report";
import type { Tool, ToolchainStatus } from "./types";

function toolRow(name: string, tool: Tool | null | undefined, note = ""): HTMLElement {
  const state = tool
    ? h("span", { className: "tool-ok" }, icon("check"), tool.version)
    : h("span", { className: "tool-missing" }, icon("alert"), "Not found");
  const where = h("span", { className: "tool-path", title: tool?.path }, tool?.path ?? note);
  return h("div", { className: "tool-row" }, h("span", { className: "tool-name" }, name), state, where);
}

function rows(status: ToolchainStatus): HTMLElement[] {
  const bundled = status.bundled_bun ? { path: "Shipped with Fusible Studio", version: status.bundled_bun } : null;
  return [
    toolRow("bun", status.bun, "Used to compile TSX and install packages"),
    toolRow("node", status.node, "Optional"),
    toolRow("tsci", status.tsci, "Fusible falls back to bun x tsci"),
    toolRow("Bundled bun", bundled, "Not included in this build"),
  ];
}

let probe: Promise<[ToolchainStatus, string]> | null = null;

function probeOnce(): Promise<[ToolchainStatus, string]> {
  probe ??= Promise.all([api.toolchainStatus(), api.appVersion().catch(() => "unknown")]);
  probe.catch(() => {
    probe = null;
  });
  return probe;
}

export async function toolchainSection(): Promise<HTMLElement> {
  const list = h("div", { className: "tools", "aria-busy": "true" }, h("p", { className: "muted" }, "Checking tools"));
  const head = h("div", { className: "section-head" }, h("h2", { className: "section-title" }, "Toolchain"));
  const section = h("section", { className: "section", "aria-labelledby": "toolchain-h" }, head, list);
  head.firstElementChild?.setAttribute("id", "toolchain-h");
  try {
    const [status, version] = await probeOnce();
    list.replaceChildren(...rows(status));
    list.removeAttribute("aria-busy");
    head.append(copyButton("Copy", () => toolchainReport(status, version)));
    const versions = `Fusible Studio ${version}, fusible ${status.fusible_version ?? "unknown"}`;
    section.append(h("p", { className: "muted" }, versions));
  } catch (error) {
    list.replaceChildren(h("p", { className: "field-error" }, message(error)));
  }
  return section;
}
