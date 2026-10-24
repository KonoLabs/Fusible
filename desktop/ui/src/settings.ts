import { h, icon } from "./dom";
import { brand } from "./projects";
import { generalSection } from "./settingsForm";
import { state } from "./store";
import { toolchainSection } from "./toolchain";

export function settingsView(): HTMLElement {
  const back = h("a", { className: "btn", href: "#", id: "back" }, icon("chevron-left"), "Projects");
  const status = h("span", { className: "save-status", role: "status", "aria-live": "polite" });
  const header = h("header", { className: "header" }, brand(), h("h1", { className: "title" }, "Settings"),
    h("div", { className: "header-actions" }, status, back));
  const main = h("main", { className: "page narrow-page" });
  if (state.settings) main.append(generalSection(state.settings, status));
  void toolchainSection().then((section) => main.append(section));
  return h("div", { className: "view" }, header, main);
}
