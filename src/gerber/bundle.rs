//! Packaging Gerber layers and drill files into zip archives.

use std::collections::BTreeMap;
use std::io::{Cursor, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

use crate::gerber::layers::{GerberError, PcbModel};

/// Map of filename to generated file contents.
pub type GerberFiles = BTreeMap<String, String>;

/// Generates all Gerber layer files and Excellon drill files from a Circuit JSON string, failing closed on bad input.
pub fn generate_gerber_files(
    circuit_json: &str,
    board_title: &str,
) -> Result<GerberFiles, GerberError> {
    let model = PcbModel::try_parse(circuit_json)?;
    let mut files = BTreeMap::new();

    files.insert("Edge_Cuts.gbr".to_string(), model.build_edge_cuts()?);
    files.insert("F_Cu.gbr".to_string(), model.build_copper_layer(true)?);
    files.insert("B_Cu.gbr".to_string(), model.build_copper_layer(false)?);
    files.insert("F_Mask.gbr".to_string(), model.build_mask_layer(true)?);
    files.insert("B_Mask.gbr".to_string(), model.build_mask_layer(false)?);
    files.insert("F_Paste.gbr".to_string(), model.build_paste_layer(true)?);
    files.insert("B_Paste.gbr".to_string(), model.build_paste_layer(false)?);
    files.insert(
        "F_Silk.gbr".to_string(),
        model.build_silkscreen_layer(true)?,
    );
    files.insert(
        "B_Silk.gbr".to_string(),
        model.build_silkscreen_layer(false)?,
    );
    files.insert(
        "drill_pth.drl".to_string(),
        model.build_drill_file(board_title)?,
    );
    if let Some(npth) = model.build_npth_drill_file(board_title)? {
        files.insert("drill_npth.drl".to_string(), npth);
    }

    Ok(files)
}

/// Compresses the result of [`generate_gerber_files`] into in-memory zip bytes, propagating a generation failure as `InvalidData`.
pub fn create_gerber_zip(
    files: &Result<GerberFiles, GerberError>,
) -> Result<Vec<u8>, std::io::Error> {
    match files {
        Ok(files) => zip_files(files),
        Err(e) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        )),
    }
}

fn zip_files(files: &GerberFiles) -> Result<Vec<u8>, std::io::Error> {
    let mut cursor = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut cursor);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for (name, content) in files {
        zip.start_file(name, options)?;
        zip.write_all(content.as_bytes())?;
    }

    zip.finish()?;
    Ok(cursor.into_inner())
}

/// Exports the complete fabrication bundle (`gerbers.zip` and `circuit.json`) to `output_dir`; nothing is written when generation fails.
pub fn export_fab_bundle(
    circuit_json: &str,
    board_title: &str,
    output_dir: &Path,
) -> Result<(), std::io::Error> {
    let files = generate_gerber_files(circuit_json, board_title)?;
    let zip_bytes = zip_files(&files)?;

    std::fs::create_dir_all(output_dir)?;
    std::fs::write(output_dir.join("gerbers.zip"), zip_bytes)?;
    std::fs::write(output_dir.join("circuit.json"), circuit_json)?;

    Ok(())
}
