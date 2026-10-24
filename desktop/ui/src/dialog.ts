import { h } from "./dom";

export type DialogAction = {
  label: string;
  primary?: boolean;
  submit?: boolean;
  onClick?: (dialog: HTMLDialogElement) => void;
};

function actionButton(action: DialogAction, dialog: HTMLDialogElement): HTMLButtonElement {
  const cls = action.primary ? "btn primary" : "btn";
  return h("button", {
    className: cls,
    type: action.submit ? "submit" : "button",
    value: action.submit ? "ok" : undefined,
    onClick: action.submit ? undefined : () => (action.onClick ? action.onClick(dialog) : dialog.close("cancel")),
  }, action.label);
}

export function makeDialog(title: string, body: Node[], actions: DialogAction[]): HTMLDialogElement {
  const dialog = h("dialog", { className: "dialog", "aria-label": title });
  const form = h("form", { method: "dialog", noValidate: true });
  const foot = h("div", { className: "dialog-actions" });
  const buttons = actions.map((action) => actionButton(action, dialog));
  foot.append(...buttons);
  form.append(h("h2", { className: "dialog-title" }, title), ...body, foot);
  dialog.append(form);
  dialog.addEventListener("close", () => dialog.remove());
  return dialog;
}

export function showDialog(dialog: HTMLDialogElement): void {
  document.body.append(dialog);
  dialog.showModal();
}

export function confirmDialog(title: string, text: string, confirm: string): Promise<boolean> {
  return new Promise((done) => {
    const body = [h("p", { className: "dialog-text" }, text)];
    const actions: DialogAction[] = [
      { label: "Cancel" },
      { label: confirm, primary: true, submit: true },
    ];
    const dialog = makeDialog(title, body, actions);
    dialog.addEventListener("close", () => done(dialog.returnValue === "ok"));
    showDialog(dialog);
    dialog.querySelector<HTMLButtonElement>(".btn.primary")?.focus();
  });
}
