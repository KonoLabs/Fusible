use std::path::Path;
use wasmtime::*;
use anyhow::{Context, Result};

pub struct MechanicalWasm {
    engine: Engine,
    module: Module,
    store: Store<()>,
    instance: Instance,
    // Raw kernel functions
    make_box: Func,
    make_cylinder: Func,
    make_sphere: Func,
    fuse: Func,
    cut: Func,
    common: Func,
    extrude: Func,
    revolve: Func,
    fillet: Func,
    chamfer: Func,
    export_step: Func,
    export_stl: Func,
    export_stl_binary: Func,
    release: Func,
    release_all: Func,
    get_shape_count: Func,
}

impl MechanicalWasm {
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        let wasm_path = Path::new("node_modules/occt-wasm/dist/occt-wasm.wasm");
        let module = Module::from_file(&engine, wasm_path)
            .with_context(|| format!("Failed to load WASM module from {}", wasm_path.display()))?;

        let mut linker = Linker::new(&engine);
        // Add the WASI imports that the module needs
        wasmtime_wasi::add_to_linker(&mut linker, |_| {}).context("Failed to add WASI to linker")?;

        let mut store = Store::new(&engine, ());
        let instance = linker
            .instantiate(&mut store, &module)
            .context("Failed to instantiate WASM module")?;

        // Get the raw kernel exports
        let kernel_exports = instance
            .get_export(&mut store, "OcctKernel")
            .context("OcctKernel export not found")?
            .into_func()
            .context("OcctKernel is not a function")?;

        // Get individual functions from the instance exports
        let get_func = |name: &str| -> Result<Func> {
            instance
                .get_export(&mut store, name)
                .with_context(|| format!("Export '{}' not found", name))?
                .into_func()
                .with_context(|| format!("Export '{}' is not a function", name))
        };

        let make_box = get_func("makeBox")?;
        let make_cylinder = get_func("makeCylinder")?;
        let make_sphere = get_func("makeSphere")?;
        let fuse = get_func("fuse")?;
        let cut = get_func("cut")?;
        let common = get_func("common")?;
        let extrude = get_func("extrude")?;
        let revolve = get_func("revolve")?;
        let fillet = get_func("fillet")?;
        let chamfer = get_func("chamfer")?;
        let export_step = get_func("exportStep")?;
        let export_stl = get_func("exportStl")?;
        let export_stl_binary = get_func("exportStlBinary")?;
        let release = get_func("release")?;
        let release_all = get_func("releaseAll")?;
        let get_shape_count = get_func("getShapeCount")?;

        Ok(Self {
            engine,
            module,
            store,
            instance,
            make_box,
            make_cylinder,
            make_sphere,
            fuse,
            cut,
            common,
            extrude,
            revolve,
            fillet,
            chamfer,
            export_step,
            export_stl,
            export_stl_binary,
            release,
            release_all,
            get_shape_count,
        })
    }

    pub fn make_box(&mut self, dx: f64, dy: f64, dz: f64) -> Result<u32> {
        let mut results = vec![Val::I32(0)];
        self.make_box
            .call(&mut self.store, &[Val::F64(dx), Val::F64(dy), Val::F64(dz)], &mut results)
            .context("Failed to call makeBox")?;
        match results[0] {
            Val::I32(id) => Ok(id as u32),
            _ => anyhow::bail!("Unexpected return type from makeBox"),
        }
    }

    pub fn make_cylinder(&mut self, radius: f64, height: f64) -> Result<u32> {
        let mut results = vec![Val::I32(0)];
        self.make_cylinder
            .call(&mut self.store, &[Val::F64(radius), Val::F64(height)], &mut results)
            .context("Failed to call makeCylinder")?;
        match results[0] {
            Val::I32(id) => Ok(id as u32),
            _ => anyhow::bail!("Unexpected return type from makeCylinder"),
        }
    }

    pub fn make_sphere(&mut self, radius: f64) -> Result<u32> {
        let mut results = vec![Val::I32(0)];
        self.make_sphere
            .call(&mut self.store, &[Val::F64(radius)], &mut results)
            .context("Failed to call makeSphere")?;
        match results[0] {
            Val::I32(id) => Ok(id as u32),
            _ => anyhow::bail!("Unexpected return type from makeSphere"),
        }
    }

    pub fn fuse(&mut self, a: u32, b: u32) -> Result<u32> {
        let mut results = vec![Val::I32(0)];
        self.fuse
            .call(&mut self.store, &[Val::I32(a as i32), Val::I32(b as i32)], &mut results)
            .context("Failed to call fuse")?;
        match results[0] {
            Val::I32(id) => Ok(id as u32),
            _ => anyhow::bail!("Unexpected return type from fuse"),
        }
    }

    pub fn cut(&mut self, a: u32, b: u32) -> Result<u32> {
        let mut results = vec![Val::I32(0)];
        self.cut
            .call(&mut self.store, &[Val::I32(a as i32), Val::I32(b as i32)], &mut results)
            .context("Failed to call cut")?;
        match results[0] {
            Val::I32(id) => Ok(id as u32),
            _ => anyhow::bail!("Unexpected return type from cut"),
        }
    }

    pub fn common(&mut self, a: u32, b: u32) -> Result<u32> {
        let mut results = vec![Val::I32(0)];
        self.common
            .call(&mut self.store, &[Val::I32(a as i32), Val::I32(b as i32)], &mut results)
            .context("Failed to call common")?;
        match results[0] {
            Val::I32(id) => Ok(id as u32),
            _ => anyhow::bail!("Unexpected return type from common"),
        }
    }

    pub fn extrude(&mut self, shape: u32, dx: f64, dy: f64, dz: f64) -> Result<u32> {
        let mut results = vec![Val::I32(0)];
        self.extrude
            .call(
                &mut self.store,
                &[
                    Val::I32(shape as i32),
                    Val::F64(dx),
                    Val::F64(dy),
                    Val::F64(dz),
                ],
                &mut results,
            )
            .context("Failed to call extrude")?;
        match results[0] {
            Val::I32(id) => Ok(id as u32),
            _ => anyhow::bail!("Unexpected return type from extrude"),
        }
    }

    pub fn revolve(
        &mut self,
        shape: u32,
        px: f64,
        py: f64,
        pz: f64,
        dx: f64,
        dy: f64,
        dz: f64,
        angle: f64,
    ) -> Result<u32> {
        let mut results = vec![Val::I32(0)];
        self.revolve
            .call(
                &mut self.store,
                &[
                    Val::I32(shape as i32),
                    Val::F64(px),
                    Val::F64(py),
                    Val::F64(pz),
                    Val::F64(dx),
                    Val::F64(dy),
                    Val::F64(dz),
                    Val::F64(angle),
                ],
                &mut results,
            )
            .context("Failed to call revolve")?;
        match results[0] {
            Val::I32(id) => Ok(id as u32),
            _ => anyhow::bail!("Unexpected return type from revolve"),
        }
    }

    pub fn fillet(&mut self, solid: u32, edges: &[u32], radius: f64) -> Result<u32> {
        // For fillet we need to pass edge IDs as a vector
        // This is simplified - the actual OCCT API uses EmbindVectorU32
        let mut results = vec![Val::I32(0)];
        // Note: The actual implementation needs vector handling
        // For now, we'll use a simplified version
        self.fillet
            .call(
                &mut self.store,
                &[Val::I32(solid as i32), Val::I32(0), Val::F64(radius)],
                &mut results,
            )
            .context("Failed to call fillet")?;
        match results[0] {
            Val::I32(id) => Ok(id as u32),
            _ => anyhow::bail!("Unexpected return type from fillet"),
        }
    }

    pub fn chamfer(&mut self, solid: u32, edges: &[u32], distance: f64) -> Result<u32> {
        let mut results = vec![Val::I32(0)];
        self.chamfer
            .call(
                &mut self.store,
                &[Val::I32(solid as i32), Val::I32(0), Val::F64(distance)],
                &mut results,
            )
            .context("Failed to call chamfer")?;
        match results[0] {
            Val::I32(id) => Ok(id as u32),
            _ => anyhow::bail!("Unexpected return type from chamfer"),
        }
    }

    pub fn export_step(&mut self, shape: u32) -> Result<String> {
        let mut results = vec![Val::I32(0)];
        self.export_step
            .call(&mut self.store, &[Val::I32(shape as i32)], &mut results)
            .context("Failed to call exportStep")?;
        // The result is a pointer to a string in WASM memory
        // We need to read it from memory
        match results[0] {
            Val::I32(ptr) => self.read_string_from_memory(ptr as u32),
            _ => anyhow::bail!("Unexpected return type from exportStep"),
        }
    }

    pub fn export_stl(&mut self, shape: u32, linear_deflection: f64, ascii: bool) -> Result<String> {
        let mut results = vec![Val::I32(0)];
        self.export_stl
            .call(
                &mut self.store,
                &[
                    Val::I32(shape as i32),
                    Val::F64(linear_deflection),
                    Val::I32(if ascii { 1 } else { 0 }),
                ],
                &mut results,
            )
            .context("Failed to call exportStl")?;
        match results[0] {
            Val::I32(ptr) => self.read_string_from_memory(ptr as u32),
            _ => anyhow::bail!("Unexpected return type from exportStl"),
        }
    }

    pub fn export_stl_binary(&mut self, shape: u32, linear_deflection: f64) -> Result<Vec<u8>> {
        let mut results = vec![Val::I32(0)];
        self.export_stl_binary
            .call(
                &mut self.store,
                &[Val::I32(shape as i32), Val::F64(linear_deflection)],
                &mut results,
            )
            .context("Failed to call exportStlBinary")?;
        match results[0] {
            Val::I32(ptr) => self.read_bytes_from_memory(ptr as u32),
            _ => anyhow::bail!("Unexpected return type from exportStlBinary"),
        }
    }

    pub fn release(&mut self, shape: u32) -> Result<()> {
        self.release
            .call(&mut self.store, &[Val::I32(shape as i32)], &mut [])
            .context("Failed to call release")?;
        Ok(())
    }

    pub fn release_all(&mut self) -> Result<()> {
        self.release_all
            .call(&mut self.store, &[], &mut [])
            .context("Failed to call releaseAll")?;
        Ok(())
    }

    pub fn shape_count(&mut self) -> Result<u32> {
        let mut results = vec![Val::I32(0)];
        self.get_shape_count
            .call(&mut self.store, &[], &mut results)
            .context("Failed to call getShapeCount")?;
        match results[0] {
            Val::I32(count) => Ok(count as u32),
            _ => anyhow::bail!("Unexpected return type from getShapeCount"),
        }
    }

    fn read_string_from_memory(&self, ptr: u32) -> Result<String> {
        let memory = self
            .instance
            .get_memory(&mut self.store.as_context_mut(), "memory")
            .context("Memory export not found")?;

        let data = memory.data(&self.store);
        let mut s = String::new();
        let mut i = ptr as usize;
        while i < data.len() {
            let byte = data[i];
            if byte == 0 {
                break;
            }
            s.push(byte as char);
            i += 1;
        }
        Ok(s)
    }

    fn read_bytes_from_memory(&self, ptr: u32) -> Result<Vec<u8>> {
        let memory = self
            .instance
            .get_memory(&mut self.store.as_context_mut(), "memory")
            .context("Memory export not found")?;

        let data = memory.data(&self.store);
        // The binary STL export returns a Uint8Array pointer
        // We need to read the length and data from the WASM memory
        // This is a simplified implementation
        let len_ptr = ptr as usize;
        let len = u32::from_le_bytes([
            data[len_ptr],
            data[len_ptr + 1],
            data[len_ptr + 2],
            data[len_ptr + 3],
        ]) as usize;

        let data_ptr = u32::from_le_bytes([
            data[len_ptr + 4],
            data[len_ptr + 5],
            data[len_ptr + 6],
            data[len_ptr + 7],
        ]) as usize;

        Ok(data[data_ptr..data_ptr + len].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mechanical_wasm_load() {
        let wasm = MechanicalWasm::new();
        assert!(wasm.is_ok());
    }
}