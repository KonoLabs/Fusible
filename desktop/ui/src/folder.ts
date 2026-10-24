import { openProject } from "./actions";
import { api, message } from "./api";
import { confirmDialog } from "./dialog";
import { openNewProject } from "./newProject";
import { baseName, suggestName } from "./path";
import { state, update } from "./store";

export async function openFolderFlow(): Promise<void> {
  try {
    const picked = await api.pickFolder("Open a Fusible project folder", state.settings?.projects_dir);
    if (!picked) return;
    const info = await api.inspectFolder(picked);
    if (info.has_manifest) return await openProject(picked, info.name ?? baseName(picked));
    const create = await confirmDialog(
      "Not a Fusible project yet",
      "This folder is not a Fusible project yet. Create one here?",
      "Create project",
    );
    if (create) openNewProject({ location: picked, createFolder: false, name: suggestName(picked) });
  } catch (error) {
    update({ failure: { title: "Could not open the folder", log: message(error) } });
  }
}
