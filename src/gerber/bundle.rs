//! Packaging Gerber layers and drill files into zip archives.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Cursor, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;

use crate::gerber::layers::PcbModel;

/// Map of filename to generated file contents.
pub type GerberFiles = BTreeMap<String, String>;

/// Generates all Gerber layer files and Excellon drill files from a Circuit JSON string.
pub fn generate_gerber_files(circuit_json: &str, board_title: &str) -> GerberFiles {
    let model = PcbModel::parse(circuit_json);
    let mut files = BTreeMap::new();

    files.insert("Edge_Cuts.gbr".to_string(), model.build_edge_cuts());
    files.insert("F_Cu.gbr".to_string(), model.build_copper_layer(true));
    files.insert("B_Cu.gbr".to_string(), model.build_copper_layer(false));
    files.insert("F_Mask.gbr".to_string(), model.build_mask_layer(true));
    files.insert("B_Mask.gbr".to_string(), model.build_mask_layer(false));
    files.insert("F_Paste.gbr".to_string(), model.build_paste_layer(true));
    files.insert("B_Paste.gbr".to_string(), model.build_paste_layer(false));
    files.insert("F_Silk.gbr".to_string(), model.build_silkscreen_layer(true));
    files.insert(
        "B_Silk.gbr".to_string(),
        model.build_silkscreen_layer(false),
    );
    files.insert(
        "drill_pth.drl".to_string(),
        model.build_drill_file(board_title),
    );
    if let Some(npth) = model.build_npth_drill_file(board_title) {
        files.insert("drill_npth.drl".to_string(), npth);
    }

    files
}

/// Compresses generated Gerber files into in-memory zip bytes.
pub fn create_gerber_zip(files: &GerberFiles) -> Result<Vec<u8>, std::io::Error> {
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

/// Exports the complete fabrication bundle (`gerbers.zip` and `circuit.json`) to `output_dir`.
pub fn export_fab_bundle(
    circuit_json: &str,
    board_title: &str,
    output_dir: &Path,
) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(output_dir)?;

    let files = generate_gerber_files(circuit_json, board_title);
    let zip_bytes = create_gerber_zip(&files)?;

    let zip_path = output_dir.join("gerbers.zip");
    std::fs::write(&zip_path, zip_bytes)?;

    let json_path = output_dir.join("circuit.json");
    let mut json_file = File::create(&json_path)?;
    json_file.write_all(circuit_json.as_bytes())?;

    Ok(())
}
