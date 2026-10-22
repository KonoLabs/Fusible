export type MechanicalOption = "lid" | "explode" | "overlay";

const DEFAULTS: Record<MechanicalOption, boolean> = { lid: true, explode: false, overlay: false };

const options: Record<MechanicalOption, boolean> = { ...DEFAULTS };

function storageKey(option: MechanicalOption): string {
  return `fusible:mechanical:${option}`;
}

/** Loads every persisted mechanical toggle from localStorage, keeping defaults when storage is unavailable. */
export function loadMechanicalOptions(): Record<MechanicalOption, boolean> {
  for (const option of Object.keys(DEFAULTS) as MechanicalOption[]) {
    try {
      const raw = localStorage.getItem(storageKey(option));
      if (raw === "1" || raw === "0") options[option] = raw === "1";
    } catch {
      options[option] = DEFAULTS[option];
    }
  }
  return { ...options };
}

/** Returns the current value of a mechanical toggle. */
export function getMechanicalOption(option: MechanicalOption): boolean {
  return options[option];
}

/** Sets a mechanical toggle and persists it, ignoring storage failures. */
export function setMechanicalOption(option: MechanicalOption, value: boolean): void {
  options[option] = value;
  try {
    localStorage.setItem(storageKey(option), value ? "1" : "0");
  } catch {
    return;
  }
}
