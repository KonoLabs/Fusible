export const FUSIBLE_SCHEMATIC_PALETTE = {
  background: "#000000",
  component_body: "#111111",
  component_outline: "#E6E6E6",
  wire: "#86A881",
  wire_crossing: "#A8B365",
  junction: "#A8B365",
  pin: "#E6E6E6",
  pin_name: "#FFFFFF",
  pin_number: "#A8B365",
  reference: "#FFFFFF",
  value: "#BDBDBD",
  label_local: "#FFFFFF",
  label_global: "#A8B365",
  label_hier: "#A8B365",
  label_background: "#111111",
  net_name: "#86A881",
  bus: "#A8B365",
  bus_junction: "#A8B365",
  grid: "#111111",
  grid_axes: "#2E2E2E",
  fields: "#BDBDBD",
  note: "#BDBDBD",
  sheet: "#2E2E2E",
  sheet_background: "#000000",
  sheet_fields: "#BDBDBD",
  sheet_filename: "#FFFFFF",
  sheet_label: "#FFFFFF",
  sheet_name: "#FFFFFF",
  cursor: "#86A881",
  aux_items: "#8A8A8A",
  erc_error: "#C55454",
  erc_warning: "#A8B365",
};

const DRAWABLE_SCHEMATIC_WARNINGS = new Set([
  "schematic_component_overlap_warning",
  "schematic_component_styling_warning",
  "schematic_element_outside_sheet_warning",
  "schematic_manual_edit_conflict_warning",
]);

function isUndrawableSchematicWarning(element: any) {
  const type = element?.type;
  return (
    typeof type === "string" &&
    type.startsWith("schematic_") &&
    type.endsWith("_warning") &&
    !DRAWABLE_SCHEMATIC_WARNINGS.has(type)
  );
}

/** Prepares Circuit JSON for the schematic viewer: drops warnings circuit-to-svg cannot draw and orphaned sheets. */
export function normalizeCircuitJsonForSchematic(input: any) {
  if (!Array.isArray(input)) return input || [];
  const circuitJson = input.filter((e: any) => !isUndrawableSchematicWarning(e));
  const sheets = circuitJson.filter((e: any) => e?.type === "schematic_sheet");
  if (!sheets.length) return circuitJson;

  const components = circuitJson.filter((e: any) => e?.type === "schematic_component");
  const componentsWithSheet = components.filter((e: any) => !!e.schematic_sheet_id);

  if (components.length > 0 && componentsWithSheet.length === 0) {
    return circuitJson.filter((e: any) => e?.type !== "schematic_sheet");
  }

  return circuitJson;
}
