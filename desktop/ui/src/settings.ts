import { h } from "./dom";
import { generalSection } from "./settingsForm";
import { state } from "./store";
import { toolchainSection } from "./toolchain";

export function settingsPage(status: HTMLElement): HTMLElement[] {
  const page = h("div", { className: "settings-page" });
  if (state.settings) page.append(generalSection(state.settings, status));
  void toolchainSection().then((section) => page.append(section));
  return [page];
}
