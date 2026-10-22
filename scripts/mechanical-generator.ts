import { OcctKernel, Vec3 } from 'occt-wasm';
import { readFileSync } from 'fs';

interface CircuitJson {
    type: string;
    outline?: { points: Vec3[] };
    components?: Array<{
        type: string;
        position: Vec3;
        size?: { width: number; height: number };
    }>;
}

async function generateMechanicalModel() {
    const kernel = await OcctKernel.init();
    const circuitJson = JSON.parse(readFileSync(0, 'utf-8')) as CircuitJson;

    // Extract board outline
    const outline = circuitJson.outline?.points;
    if (!outline || outline.length < 3) {
        throw new Error('No valid board outline found in circuit.json');
    }

    // Create base box from outline
    const minX = Math.min(...outline.map(p => p.x));
    const maxX = Math.max(...outline.map(p => p.x));
    const minY = Math.min(...outline.map(p => p.y));
    const maxY = Math.max(...outline.map(p => p.y));
    const width = maxX - minX;
    const height = maxY - minY;

    // Add wall thickness
    const wallThickness = 2.0; // mm
    const enclosureHeight = 20.0; // mm

    // Create enclosure
    const enclosure = kernel.makeBox(width + 2 * wallThickness, height + 2 * wallThickness, enclosureHeight);

    // Add component cutouts
    circuitJson.components?.forEach(component => {
        if (component.type === 'usb_c') {
            const cutout = kernel.makeBox(12, 6, 5);
            kernel.translate(cutout, component.position.x, component.position.y, 0);
            kernel.cut(enclosure, cutout);
        }
    });

    // Export to STEP
    kernel.writeStep(enclosure, 'output.step');
    console.log('Mechanical model generated successfully');
}

generateMechanicalModel().catch(console.error);
