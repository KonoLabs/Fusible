import { api, message } from "./api";
import { openProject } from "./actions";
import { makeDialog, showDialog } from "./dialog";
import { h, icon } from "./dom";
import { checkbox, currencySelect, errorLine, field, pathRow, setError } from "./fields";
import { finalPath } from "./path";
import { state } from "./store";
import { validateName } from "./validate";

export type NewProjectOptions = { name?: string; location?: string; createFolder?: boolean };

type Parts = {
  name: HTMLInputElement;
  nameError: HTMLElement;
  location: HTMLInputElement;
  folder: HTMLInputElement;
  folderName: HTMLElement;
  currency: HTMLSelectElement;
  preview: HTMLOutputElement;
  formError: HTMLElement;
  create: HTMLButtonElement;
};

function buildParts(opts: NewProjectOptions): { nodes: Node[]; parts: Omit<Parts, "create"> } {
  const settings = state.settings;
  const name = h("input", {
    id: "np-name", className: "input", type: "text", value: opts.name ?? "", placeholder: "My board",
    spellcheck: false, autocomplete: "off", maxLength: 120,
  });
  const nameError = errorLine("np-name-error");
  const row = pathRow("np-location", opts.location ?? settings?.projects_dir ?? "", () => void browse(row));
  const location = row.querySelector("input") as HTMLInputElement;
  const folderName = h("code", null, opts.name || "name");
  const folderLabel = checkbox("np-folder", opts.createFolder ?? true, "Create a folder named ", folderName);
  const folder = folderLabel.querySelector("input") as HTMLInputElement;
  const currency = currencySelect("np-currency", settings?.default_currency ?? "EUR");
  const preview = h("output", { className: "final-path", htmlFor: "np-name np-location np-folder" });
  const formError = errorLine("np-form-error");
  const nodes = [
    field("np-name", "Name", name, nameError),
    field("np-location", "Location", row),
    folderLabel,
    field("np-currency", "Currency", currency),
    h("div", { className: "final" }, h("span", { className: "field-label" }, "Final path"), preview),
    formError,
  ];
  return { nodes, parts: { name, nameError, location, folder, folderName, currency, preview, formError } };
}

async function browse(row: HTMLElement): Promise<void> {
  const input = row.querySelector("input") as HTMLInputElement;
  const picked = await api.pickFolder("Choose a location", input.value || undefined).catch(() => null);
  if (picked) {
    input.value = picked;
    input.dispatchEvent(new Event("input", { bubbles: true }));
  }
}

function refresh(parts: Parts, touched: { name: boolean }): void {
  const nameProblem = validateName(parts.name.value);
  const locationProblem = parts.location.value.trim() ? null : "Choose a location.";
  setError(parts.nameError, touched.name ? nameProblem : null);
  parts.name.setAttribute("aria-invalid", String(touched.name && Boolean(nameProblem)));
  parts.folderName.textContent = parts.name.value.trim() || "name";
  const path = finalPath(parts.location.value, parts.name.value, parts.folder.checked);
  parts.preview.textContent = path || "Choose a location";
  parts.create.disabled = Boolean(nameProblem || locationProblem);
}

async function submit(parts: Parts, dialog: HTMLDialogElement): Promise<void> {
  setError(parts.formError, null);
  parts.create.disabled = true;
  parts.create.replaceChildren(icon("loader", "spin"), "Creating");
  try {
    const project = await api.createProject({
      parent: parts.location.value.trim(),
      name: parts.name.value.trim(),
      create_folder: parts.folder.checked,
      currency: parts.currency.value,
    });
    dialog.close("created");
    await openProject(project.path, project.name);
  } catch (error) {
    setError(parts.formError, message(error));
    parts.create.textContent = "Create project";
    parts.create.disabled = false;
  }
}

export function openNewProject(opts: NewProjectOptions = {}): void {
  const { nodes, parts: base } = buildParts(opts);
  const dialog = makeDialog("New project", nodes, [
    { label: "Cancel" },
    { label: "Create project", primary: true, submit: true },
  ]);
  const create = dialog.querySelector<HTMLButtonElement>(".btn.primary") as HTMLButtonElement;
  const parts: Parts = { ...base, create };
  const touched = { name: Boolean(opts.name) };
  dialog.addEventListener("input", () => {
    touched.name ||= document.activeElement === parts.name;
    refresh(parts, touched);
  });
  dialog.addEventListener("change", () => refresh(parts, touched));
  dialog.querySelector("form")?.addEventListener("submit", (event) => {
    event.preventDefault();
    if (!create.disabled) void submit(parts, dialog);
  });
  refresh(parts, touched);
  showDialog(dialog);
  parts.name.focus();
}
