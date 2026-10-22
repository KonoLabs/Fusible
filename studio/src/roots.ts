import type { ReactElement } from "react";
import { createRoot, type Root } from "react-dom/client";

export interface RootSlot {
  root: Root | null;
  container: HTMLElement | null;
}

/** Creates an empty slot that keeps one React root per viewer. */
export function rootSlot(): RootSlot {
  return { root: null, container: null };
}

/** Renders `element` into `container`, reusing the slot's root and remounting when the container changes. */
export function renderIntoSlot(slot: RootSlot, container: HTMLElement, element: ReactElement): void {
  if (!container) return;
  if (slot.container !== container || !slot.root) {
    slot.root?.unmount();
    container.innerHTML = "";
    slot.root = createRoot(container);
    slot.container = container;
  }
  slot.root.render(element);
}
