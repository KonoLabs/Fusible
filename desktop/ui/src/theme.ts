import type { Theme } from "./types";

const media = window.matchMedia("(prefers-color-scheme: light)");
let chosen: Theme = "system";

function resolved(): "dark" | "light" {
  if (chosen === "system") return media.matches ? "light" : "dark";
  return chosen;
}

export function applyTheme(theme: Theme): void {
  chosen = theme;
  document.documentElement.dataset.theme = resolved();
}

media.addEventListener("change", () => applyTheme(chosen));
