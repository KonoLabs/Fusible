import { copyText } from "./actions";
import { h, icon } from "./dom";

export function copyButton(text: string, getText: () => string): HTMLButtonElement {
  const button = h("button", { className: "btn", type: "button" });
  const idle = () => button.replaceChildren(icon("copy"), text);
  idle();
  button.addEventListener("click", async () => {
    const ok = await copyText(getText());
    button.replaceChildren(icon(ok ? "check" : "alert"), ok ? "Copied" : "Copy failed");
    setTimeout(idle, 1800);
  });
  return button;
}
