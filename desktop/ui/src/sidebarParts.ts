import { h, icon } from "./dom";
import { state, update } from "./store";

export function brand(): HTMLElement {
  const name = h("span", { className: "brand-name" }, "FUSIBLE");
  return h("div", { className: "brand" }, icon("cpu", "brand-logo"), name);
}

export function search(): HTMLElement {
  const input = h("input", {
    className: "input search-input", id: "search", type: "search", placeholder: "Search projects",
    value: state.query, autocomplete: "off", spellcheck: false, "aria-label": "Search projects",
  });
  input.addEventListener("input", () => update({ query: input.value }));
  return h("div", { className: "search" }, icon("search"), input);
}

export function toolchainLine(): HTMLElement {
  const tools = state.toolchain;
  const ok = tools ? Boolean(tools.bun || tools.bundled_bun) : null;
  const text = ok === null ? "Checking tools" : ok ? "Toolchain ready" : "bun not found";
  const kind = ok === null ? "busy" : ok ? "ready" : "missing";
  return h("a", { className: "status-line", href: "#settings", title: "Toolchain status" },
    h("span", { className: `dot ${kind}`, "aria-hidden": "true" }), text);
}

export function settingsLink(): HTMLElement {
  const active = state.view === "settings";
  return h("a", {
    className: active ? "nav-link active" : "nav-link", href: "#settings",
    "aria-current": active ? "page" : false,
  }, icon("gear"), "Settings");
}

