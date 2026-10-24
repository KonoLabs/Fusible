import { h } from "./dom";
import { CURRENCIES } from "./labels";

export function field(id: string, text: string, control: Node, ...extra: (Node | null)[]): HTMLDivElement {
  const row = h("div", { className: "field" });
  row.append(h("label", { htmlFor: id, className: "field-label" }, text), control);
  for (const node of extra) if (node) row.append(node);
  return row;
}

export function currencySelect(id: string, value: string): HTMLSelectElement {
  const select = h("select", { id, className: "input" });
  const codes = CURRENCIES.includes(value) ? CURRENCIES : [value, ...CURRENCIES];
  for (const code of codes) select.append(h("option", { value: code, selected: code === value }, code));
  return select;
}

export function checkbox(id: string, checked: boolean, ...text: (Node | string)[]): HTMLLabelElement {
  const input = h("input", { id, type: "checkbox", className: "check", checked });
  return h("label", { className: "check-row", htmlFor: id }, input, h("span", null, ...text));
}

export function errorLine(id: string): HTMLParagraphElement {
  return h("p", { id, className: "field-error", role: "alert", hidden: true });
}

export function setError(line: HTMLElement, text: string | null): void {
  line.textContent = text ?? "";
  line.hidden = !text;
}

export function pathRow(id: string, value: string, browse: () => void): HTMLDivElement {
  const input = h("input", { id, className: "input", type: "text", value, spellcheck: false, autocomplete: "off" });
  const button = h("button", { className: "btn", type: "button", onClick: browse }, "Browse");
  return h("div", { className: "path-row" }, input, button);
}
