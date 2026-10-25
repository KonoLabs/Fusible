import { themeColor } from "./theme";

/** The schematic viewer palette, read from the theme at call time. */
export function fusibleSchematicPalette() {
  return {
    background: themeColor("bg"),
    component_body: themeColor("surface"),
    component_outline: themeColor("silver"),
    wire: themeColor("green"),
    wire_crossing: themeColor("olive"),
    junction: themeColor("olive"),
    pin: themeColor("silver"),
    pin_name: themeColor("text"),
    pin_number: themeColor("olive"),
    reference: themeColor("text"),
    value: themeColor("muted"),
    label_local: themeColor("text"),
    label_global: themeColor("olive"),
    label_hier: themeColor("olive"),
    label_background: themeColor("surface"),
    net_name: themeColor("green"),
    bus: themeColor("olive"),
    bus_junction: themeColor("olive"),
    grid: themeColor("surface"),
    grid_axes: themeColor("border"),
    fields: themeColor("muted"),
    note: themeColor("muted"),
    sheet: themeColor("border"),
    sheet_background: themeColor("bg"),
    sheet_fields: themeColor("muted"),
    sheet_filename: themeColor("text"),
    sheet_label: themeColor("text"),
    sheet_name: themeColor("text"),
    cursor: themeColor("green"),
    aux_items: themeColor("dim"),
    erc_error: themeColor("alert"),
    erc_warning: themeColor("olive"),
  };
}

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
