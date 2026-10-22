import React, { useState, useEffect, useRef } from "react";
import { PCBViewer } from "@tscircuit/pcb-viewer";
import { rootSlot, renderIntoSlot } from "./roots";
import { useContextMenuIntercept } from "./contextMenu";
import { resolveComponentRefdes } from "./refdes";

const pcbSlot = rootSlot();

function ResponsivePcbViewer({
  circuitJson,
  onSelectComponent,
}: {
  circuitJson: any;
  onSelectComponent?: (name: string) => void;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [viewerHeight, setViewerHeight] = useState(600);

  useEffect(() => {
    try {
      localStorage.setItem("pcb-viewer:rendering-engine", "canvas");
    } catch {}
    const parent = containerRef.current?.parentElement || containerRef.current;
    if (!parent) return;
    const updateSize = () => {
      const h = parent.clientHeight || window.innerHeight - 52;
      if (h > 50) {
        setViewerHeight(h);
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

  useContextMenuIntercept(circuitJson || [], onSelectComponent);

  return React.createElement(
    "div",
    {
      ref: containerRef,
      style: { width: "100%", height: "100%", position: "relative", overflow: "hidden" },
    },
    React.createElement(PCBViewer, {
      circuitJson: circuitJson || [],
      height: viewerHeight,
      renderer: "canvas",
      allowEditing: false,
      onViewSchematicComponent: (event: any) => {
        const refdes = event?.refdes || event?.source_component_id;
        if (refdes && onSelectComponent) {
          onSelectComponent(resolveComponentRefdes(circuitJson || [], refdes));
        }
      },
    })
  );
}

export function renderPcbViewer(
  container: HTMLElement,
  circuitJson: any,
  onSelectComponent?: (name: string) => void
) {
  const props = { circuitJson: circuitJson || [], onSelectComponent };
  renderIntoSlot(pcbSlot, container, React.createElement(ResponsivePcbViewer, props));
}
