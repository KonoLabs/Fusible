/** Resolves any source, schematic, pcb or cad component id to its refdes, falling back to the id itself. */
export function resolveComponentRefdes(circuitJson: any[], id: string): string {
  if (!id) return "";
  if (!Array.isArray(circuitJson)) return id;
  const directSource = circuitJson.find(
    (e: any) => e.type === "source_component" && (e.name === id || e.source_component_id === id)
  );
  if (directSource?.name) return directSource.name;

  const schComp = circuitJson.find(
    (e: any) => e.type === "schematic_component" && (e.schematic_component_id === id || e.source_component_id === id)
  );
  if (schComp?.source_component_id) {
    const src = circuitJson.find(
      (e: any) => e.type === "source_component" && e.source_component_id === schComp.source_component_id
    );
    if (src?.name) return src.name;
  }

  const pcbComp = circuitJson.find(
    (e: any) => e.type === "pcb_component" && (e.pcb_component_id === id || e.source_component_id === id)
  );
  if (pcbComp?.source_component_id) {
    const src = circuitJson.find(
      (e: any) => e.type === "source_component" && e.source_component_id === pcbComp.source_component_id
    );
    if (src?.name) return src.name;
  }

  const cadComp = circuitJson.find(
    (e: any) =>
      e.type === "cad_component" &&
      (e.cad_component_id === id || e.source_component_id === id || e.pcb_component_id === id)
  );
  if (cadComp?.source_component_id) {
    const src = circuitJson.find(
      (e: any) => e.type === "source_component" && e.source_component_id === cadComp.source_component_id
    );
    if (src?.name) return src.name;
  }
  if (cadComp?.pcb_component_id) {
    const pcb = circuitJson.find(
      (e: any) => e.type === "pcb_component" && e.pcb_component_id === cadComp.pcb_component_id
    );
    if (pcb?.source_component_id) {
      const src = circuitJson.find(
        (e: any) => e.type === "source_component" && e.source_component_id === pcb.source_component_id
      );
      if (src?.name) return src.name;
    }
  }

  return id;
}

/** Walks up a hit 3D object and returns the refdes of the component it belongs to, or an empty string. */
export function determineRefdesFrom3dObject(circuitJson: any[], object: any): string {
  let curr: any = object;
  let pcbId = "";
  let cadId = "";
  let srcId = "";
  let refdes = "";
  while (curr) {
    if (curr.visible === false) return "";
    if (curr.userData?.isReferenceObject) return "";
    if (!pcbId && curr.userData?.pcb_component_id) pcbId = curr.userData.pcb_component_id;
    if (!cadId && curr.userData?.cad_component_id) cadId = curr.userData.cad_component_id;
    if (!srcId && curr.userData?.source_component_id) srcId = curr.userData.source_component_id;
    if (!refdes && curr.userData?.refdes) refdes = curr.userData.refdes;
    if (!refdes && curr.userData?.name) refdes = curr.userData.name;
    curr = curr.parent;
  }
  const candidate = srcId || pcbId || cadId || refdes;
  if (!candidate) return "";
  const resolved = resolveComponentRefdes(circuitJson, candidate);
  if (resolved && circuitJson.some((e: any) => e.type === "source_component" && e.name === resolved)) {
    return resolved;
  }
  const prefixes = ["cad_", "pcb_", "source_", "schematic_"];
  if (resolved && !prefixes.some((prefix) => resolved.startsWith(prefix))) {
    return resolved;
  }
  return "";
}
