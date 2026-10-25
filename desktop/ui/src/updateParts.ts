import { api } from "./api";
import { h, icon } from "./dom";
import { laterNow, installNow, restartNow, retryNow } from "./updater";
import { releaseUrl } from "./updateText";

export function button(label: string, onClick: () => void, primary = false, id = ""): HTMLButtonElement {
  const className = primary ? "btn primary" : "btn";
  return h("button", { className, type: "button", onClick, id: id || false }, label);
}

export function link(label: string, url: string): HTMLAnchorElement {
  const anchor = h("a", { className: "link", href: url, rel: "noreferrer" }, label);
  anchor.addEventListener("click", (event) => {
    event.preventDefault();
    void api.openUrl(url).catch(() => window.open(url, "_blank"));
  });
  return anchor;
}

export function notesLink(version: string): HTMLAnchorElement {
  return link("Release notes", releaseUrl(version));
}

export function progressBar(percent: number | null, label: string): HTMLProgressElement {
  const bar = h("progress", { className: "progress", max: 100, "aria-label": label });
  if (percent !== null) bar.value = percent;
  return bar;
}

export function installButton(id = ""): HTMLButtonElement {
  return button("Install", () => void installNow(), true, id);
}

export function restartButton(id = ""): HTMLButtonElement {
  return button("Restart now", () => void restartNow(), true, id);
}

export function laterButton(): HTMLButtonElement {
  return button("Later", laterNow);
}

export function retryButton(id = ""): HTMLButtonElement {
  const retry = button("Try again", retryNow, false, id);
  retry.prepend(icon("reload"));
  return retry;
}
