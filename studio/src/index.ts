import {
  convertCircuitJsonToSchematicSvg,
  convertCircuitJsonToPcbSvg,
  colorMap,
} from "circuit-to-svg";

export {
  convertCircuitJsonToSchematicSvg,
  convertCircuitJsonToPcbSvg,
  colorMap,
};

export const Tscircuit = {
  convertCircuitJsonToSchematicSvg,
  convertCircuitJsonToPcbSvg,
  colorMap,
};

declare global {
  interface Window {
    Tscircuit: typeof Tscircuit;
  }
}

if (typeof window !== "undefined") {
  window.Tscircuit = Tscircuit;
}

export default Tscircuit;
