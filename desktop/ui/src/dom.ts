import icons from "../assets/icons.json";

type Child = Node | string | null | false | undefined;
type Props = Record<string, unknown>;

export type IconName = keyof typeof icons;

export function icon(name: IconName, className = ""): SVGElement {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("aria-hidden", "true");
  svg.setAttribute("class", `icon ${className}`.trim());
  svg.innerHTML = icons[name];
  return svg;
}

function assign(node: HTMLElement, props: Props): void {
  for (const [key, value] of Object.entries(props)) {
    if (value === undefined || value === false) continue;
    if (key.startsWith("on") && typeof value === "function") {
      node.addEventListener(key.slice(2).toLowerCase(), value as EventListener);
    } else if (key in node && key !== "list") {
      (node as unknown as Props)[key] = value;
    } else {
      node.setAttribute(key, value === true ? "" : String(value));
    }
  }
}

export function h<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  props: Props | null = null,
  ...children: Child[]
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (props) assign(node, props);
  for (const child of children) {
    if (child) node.append(child);
  }
  return node;
}

export function clear(node: Element): void {
  node.replaceChildren();
}

export function label(text: string, shortcut?: string): Child[] {
  return [text, shortcut ? h("kbd", null, shortcut) : null];
}
