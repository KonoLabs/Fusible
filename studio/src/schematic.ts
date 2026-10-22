import { convertCircuitJsonToSchematicSvg } from "circuit-to-svg";

export const FUSIBLE_SCHEMATIC_PALETTE = {
  background: "#000000",
  component_body: "#1F1F1F",
  component_outline: "#86A881",
  wire: "#38bdf8",
  wire_crossing: "#7dd3fc",
  junction: "#38bdf8",
  pin: "#86A881",
  pin_name: "#FFFFFF",
  pin_number: "#A8B365",
  reference: "#86A881",
  value: "#FFFFFF",
  label_local: "#FFFFFF",
  label_global: "#f59e0b",
  label_hier: "#A8B365",
  label_background: "#1F1F1F",
  net_name: "#38bdf8",
  bus: "#818cf8",
  bus_junction: "#818cf8",
  grid: "#1F1F1F",
  grid_axes: "#353535",
  fields: "#cbd5e1",
  note: "#93c5fd",
  sheet: "#353535",
  sheet_background: "#000000",
  sheet_fields: "#94a3b8",
  sheet_filename: "#FFFFFF",
  sheet_label: "#FFFFFF",
  sheet_name: "#FFFFFF",
  cursor: "#86A881",
  aux_items: "#64748b",
  erc_error: "#C55454",
  erc_warning: "#f59e0b",
};

export function normalizeCircuitJsonForSchematic(circuitJson: any) {
  if (!Array.isArray(circuitJson)) return circuitJson || [];
  const sheets = circuitJson.filter((e: any) => e?.type === "schematic_sheet");
  if (!sheets.length) return circuitJson;

  const components = circuitJson.filter((e: any) => e?.type === "schematic_component");
  const componentsWithSheet = components.filter((e: any) => !!e.schematic_sheet_id);

  if (components.length > 0 && componentsWithSheet.length === 0) {
    return circuitJson.filter((e: any) => e?.type !== "schematic_sheet");
  }

  return circuitJson;
}

export function renderFusibleSchematicSvg(circuitJson: any, options: any = {}) {
  const normalized = normalizeCircuitJsonForSchematic(circuitJson);
  return convertCircuitJsonToSchematicSvg(normalized, {
    ...options,
    colorOverrides: {
      schematic: FUSIBLE_SCHEMATIC_PALETTE,
      ...(options?.colorOverrides || {}),
    },
  });
}
