import React, { useState, useEffect, useRef } from "react";
import { SchematicViewer } from "@tscircuit/schematic-viewer";
import { rootSlot, renderIntoSlot } from "./roots";
import { resolveComponentRefdes } from "./refdes";
import { themeColor } from "./theme";
import { fusibleSchematicPalette, normalizeCircuitJsonForSchematic } from "./schematic";

const schematicSlot = rootSlot();

function ResponsiveSchematicViewer({
  circuitJson,
  onSelectComponent,
}: {
  circuitJson: any;
  onSelectComponent?: (name: string) => void;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({ width: 800, height: 600 });
  const normalizedCircuit = React.useMemo(
    () => normalizeCircuitJsonForSchematic(circuitJson),
    [circuitJson]
  );

  useEffect(() => {
    const parent = containerRef.current?.parentElement || containerRef.current;
    if (!parent) return;
    const updateSize = () => {
      const w = parent.clientWidth || window.innerWidth;
      const h = parent.clientHeight || window.innerHeight - 52;
      if (w > 50 && h > 50) {
        setDimensions({ width: w, height: h });
      }
    };
    updateSize();
    const ro = new ResizeObserver(updateSize);
    ro.observe(parent);
    window.addEventListener("resize", updateSize);
    return () => {
      ro.disconnect();
      window.removeEventListener("resize", updateSize);
    };
  }, []);

  return React.createElement(
    "div",
    {
      ref: containerRef,
      style: { width: "100%", height: "100%", position: "relative", overflow: "hidden" },
    },
    React.createElement(SchematicViewer, {
      key: `${dimensions.width}x${dimensions.height}`,
      circuitJson: normalizedCircuit || [],
      containerStyle: { width: "100%", height: "100%", backgroundColor: themeColor("bg") },
      colorOverrides: {
        schematic: fusibleSchematicPalette(),
      },
      onSchematicComponentClicked: (event: any) => {
        const id = event?.schematicComponentId || event?.refdes;
        if (id && onSelectComponent) {
          onSelectComponent(resolveComponentRefdes(normalizedCircuit || [], id));
        }
      },
      onViewPcbComponent: (event: any) => {
        const refdes = event?.refdes;
        if (refdes && onSelectComponent) {
          onSelectComponent(resolveComponentRefdes(normalizedCircuit || [], refdes));
        }
      },
    })
  );
}

export function renderSchematicViewer(
  container: HTMLElement,
  circuitJson: any,
  onSelectComponent?: (name: string) => void
) {
  const props = { circuitJson: circuitJson || [], onSelectComponent };
  renderIntoSlot(schematicSlot, container, React.createElement(ResponsiveSchematicViewer, props));
}
