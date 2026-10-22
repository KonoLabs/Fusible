import { useEffect } from "react";
import { resolveComponentRefdes } from "./refdes";

type CircuitElements = Parameters<typeof resolveComponentRefdes>[0];
type Select = (name: string) => void;

const MENU_ITEMS = 'button, [role="menuitem"]';

/** Returns the refdes candidates named by an upstream context menu entry ("<ref> on Schematic", 'Hide "<ref>"'). */
function menuRefdes(circuitJson: CircuitElements, text: string): string[] {
  const candidates: string[] = [];
  if (text.includes("on Schematic")) candidates.push(text.replace("on Schematic", "").replace(/[↗︎\s]/g, "").trim());
  const hidden = text.match(/Hide\s*"([^"]+)"/)?.[1];
  if (hidden) candidates.push(hidden);
  return candidates.filter(Boolean).map((ref) => resolveComponentRefdes(circuitJson, ref)).filter(Boolean);
}

function addedMenus(mutations: MutationRecord[]): Element[] {
  const nodes = mutations.flatMap((mutation) => Array.from(mutation.addedNodes));
  const elements = nodes.filter((node): node is HTMLElement => node instanceof HTMLElement);
  const menus = elements.map((el) => (el.getAttribute("role") === "menu" ? el : el.querySelector('[role="menu"]')));
  return menus.filter((menu): menu is Element => !!menu);
}

function firstMenuRefdes(circuitJson: CircuitElements, menus: Element[]): string | undefined {
  const items = menus.flatMap((menu) => Array.from(menu.querySelectorAll(MENU_ITEMS)));
  for (const item of items) {
    const [ref] = menuRefdes(circuitJson, item.textContent || "");
    if (ref) return ref;
  }
  return undefined;
}

/** Selects the component an upstream viewer context menu refers to, when the menu opens or an entry is clicked. */
export function useContextMenuIntercept(circuitJson: CircuitElements, onSelectComponent?: Select): void {
  useEffect(() => {
    if (!onSelectComponent) return;
    const handleDocumentClick = (e: MouseEvent) => {
      const item = (e.target as HTMLElement | null)?.closest(MENU_ITEMS);
      if (item) menuRefdes(circuitJson || [], item.textContent || "").forEach(onSelectComponent);
    };
    const observer = new MutationObserver((mutations) => {
      const ref = firstMenuRefdes(circuitJson || [], addedMenus(mutations));
      if (ref) onSelectComponent(ref);
    });
    document.addEventListener("click", handleDocumentClick, true);
    observer.observe(document.body, { childList: true, subtree: true });
    return () => {
      document.removeEventListener("click", handleDocumentClick, true);
      observer.disconnect();
    };
  }, [circuitJson, onSelectComponent]);
}
