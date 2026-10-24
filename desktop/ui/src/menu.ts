import { h, icon, type IconName } from "./dom";

export type MenuItem = { label: string; icon: IconName; danger?: boolean; run: () => void };

let current: { menu: HTMLElement; anchor: HTMLElement } | null = null;

export function closeMenu(restoreFocus = false): void {
  if (!current) return;
  const { menu, anchor } = current;
  current = null;
  menu.remove();
  anchor.setAttribute("aria-expanded", "false");
  if (restoreFocus) anchor.focus();
}

export function menuIsOpen(): boolean {
  return current !== null;
}

function move(menu: HTMLElement, step: number): void {
  const items = [...menu.querySelectorAll<HTMLElement>("[role=menuitem]")];
  const at = items.indexOf(document.activeElement as HTMLElement);
  items[(at + step + items.length) % items.length]?.focus();
}

function onKey(event: KeyboardEvent, menu: HTMLElement): void {
  const steps: Record<string, number> = { ArrowDown: 1, ArrowUp: -1 };
  if (event.key in steps) move(menu, steps[event.key]);
  else if (event.key === "Escape") closeMenu(true);
  else if (event.key === "Tab") closeMenu();
  else return;
  event.preventDefault();
  event.stopPropagation();
}

function place(menu: HTMLElement, anchor: HTMLElement): void {
  const box = anchor.getBoundingClientRect();
  menu.style.top = `${box.bottom + 4}px`;
  menu.style.right = `${document.documentElement.clientWidth - box.right}px`;
}

function entry(item: MenuItem): HTMLButtonElement {
  const cls = item.danger ? "menu-item danger" : "menu-item";
  const run = () => {
    closeMenu();
    item.run();
  };
  return h("button", { className: cls, role: "menuitem", type: "button", onClick: run }, icon(item.icon), item.label);
}

export function openMenu(anchor: HTMLElement, items: MenuItem[]): void {
  closeMenu();
  const menu = h("div", { className: "menu", role: "menu" }, ...items.map(entry));
  menu.addEventListener("keydown", (event) => onKey(event, menu));
  document.body.append(menu);
  place(menu, anchor);
  anchor.setAttribute("aria-expanded", "true");
  current = { menu, anchor };
  menu.querySelector<HTMLElement>("[role=menuitem]")?.focus();
}

document.addEventListener("pointerdown", (event) => {
  if (current && !current.menu.contains(event.target as Node) && !current.anchor.contains(event.target as Node)) {
    closeMenu();
  }
});
window.addEventListener("resize", () => closeMenu());
