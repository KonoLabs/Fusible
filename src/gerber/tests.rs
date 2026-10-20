use std::io::Read;

use super::*;
use crate::gerber::apertures::Aperture;
use crate::gerber::coordinates::Coord;

#[test]
fn fixed_point_coordinate_formatting() {
    let c1 = Coord::from_mm(1.25);
    assert_eq!(c1.format_gerber(), "1250000");
    assert_eq!(c1.format_drill(), "1.2500");

    let c2 = Coord::from_mm(-3.4);
    assert_eq!(c2.format_gerber(), "-3400000");
    assert_eq!(c2.format_drill(), "-3.4000");

    let c0 = Coord::from_mm(0.0);
    assert_eq!(c0.format_gerber(), "0");
    assert_eq!(c0.format_drill(), "0.0000");
}

#[test]
fn aperture_dictionary_deduplicates_and_allocates_d_codes() {
    let mut dict = ApertureDictionary::new();

    let ap1 = Aperture::Circle {
        diameter_nm: 200_000,
    };
    let id1 = dict.intern(ap1.clone());
    assert_eq!(id1, 10);

    let ap2 = Aperture::Rectangle {
        width_nm: 1_000_000,
        height_nm: 500_000,
    };
    let id2 = dict.intern(ap2);
    assert_eq!(id2, 11);

    // Duplicate circle should reuse D10
    let id3 = dict.intern(ap1);
    assert_eq!(id3, 10);

    let mut out = String::new();
    dict.write_definitions(&mut out);
    assert!(out.contains("%ADD10C,0.200000*%"));
    assert!(out.contains("%ADD11R,1.000000X0.500000*%"));
}

#[test]
fn gerber_writer_structure_and_termination() {
    let mut writer = GerberWriter::new("Top Copper", "Copper,L1,Top", true);
    let mut dict = ApertureDictionary::new();
    let ap = dict.intern(Aperture::Circle {
        diameter_nm: 250_000,
    });
    writer.write_apertures(&dict);

    writer.select_aperture(ap);
    writer.move_to(Coord::from_mm(0.0), Coord::from_mm(0.0));
    writer.draw_to(Coord::from_mm(10.0), Coord::from_mm(10.0));
    writer.flash(Coord::from_mm(5.0), Coord::from_mm(5.0));

    let content = writer.finish();
    assert!(content.contains("%FSLAX46Y46*%"));
    assert!(content.contains("%MOMM*%"));
    assert!(content.contains("%LPD*%"));
    assert!(content.contains("%TF.FileFunction,Copper,L1,Top*%"));
    assert!(content.contains("D10*"));
    assert!(content.contains("X0Y0D02*"));
    assert!(content.contains("X10000000Y10000000D01*"));
    assert!(content.contains("X5000000Y5000000D03*"));
    assert!(content.ends_with("M02*\n"));
}

#[test]
fn excellon_writer_structure_and_tool_registration() {
    let mut writer = ExcellonWriter::new("MyBoard", "Plated,1,2,PTH");
    let t1 = writer.register_tool(0.8);
    let t2 = writer.register_tool(0.3);
    // Duplicate diameter should reuse tool
    let t3 = writer.register_tool(0.8);
    assert_eq!(t1, t3);
    assert_ne!(t1, t2);

    writer.add_hole(t1, Coord::from_mm(12.5), Coord::from_mm(25.0));
    writer.add_hole(t2, Coord::from_mm(0.0), Coord::from_mm(5.0));

    let content = writer.finish();
    assert!(content.contains("M48"));
    assert!(content.contains("METRIC,LZ"));
    assert!(content.contains("T01C0.8000"));
    assert!(content.contains("T02C0.3000"));
    assert!(content.contains("G90"));
    assert!(content.contains("G05"));
    assert!(content.contains("T01\nX12.5000Y25.0000"));
    assert!(content.contains("T02\nX0.0000Y5.0000"));
    assert!(content.ends_with("M30\n"));
}

#[test]
fn generates_complete_gerber_bundle_from_circuit_json() {
    let circuit_json = r#"[
        {
            "type": "pcb_board",
            "width": 50,
            "height": 40,
            "center": { "x": 0, "y": 0 }
        },
        {
            "type": "pcb_smtpad",
            "layer": "top",
            "x": 2.5,
            "y": 1.0,
            "width": 1.2,
            "height": 0.6,
            "shape": "rect"
        },
        {
            "type": "pcb_plated_hole",
            "x": -5.0,
            "y": 0.0,
            "hole_diameter": 0.8,
            "outer_diameter": 1.6
        },
        {
            "type": "pcb_trace",
            "route": [
                { "route_type": "wire", "x": 0.0, "y": 0.0, "width": 0.2, "layer": "top" },
                { "route_type": "wire", "x": 2.5, "y": 1.0, "width": 0.2, "layer": "top" }
            ]
        }
    ]"#;

    let files = generate_gerber_files(circuit_json, "TestBoard");
    assert!(files.contains_key("Edge_Cuts.gbr"));
    assert!(files.contains_key("F_Cu.gbr"));
    assert!(files.contains_key("B_Cu.gbr"));
    assert!(files.contains_key("F_Mask.gbr"));
    assert!(files.contains_key("B_Mask.gbr"));
    assert!(files.contains_key("drill_pth.drl"));

    let edge_cuts = &files["Edge_Cuts.gbr"];
    assert!(edge_cuts.contains("Profile,NP"));
    assert!(edge_cuts.contains("M02*"));

    let f_cu = &files["F_Cu.gbr"];
    assert!(f_cu.contains("Copper,L1,Top"));
    assert!(f_cu.contains("D03*")); // Flashed pad
    assert!(f_cu.contains("D01*")); // Drawn trace

    // Test zip archive creation and extraction
    let zip_bytes = create_gerber_zip(&files).expect("zip creation");
    let cursor = std::io::Cursor::new(zip_bytes);
    let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");

    let mut edge_entry = archive.by_name("Edge_Cuts.gbr").expect("entry exists");
    let mut decompressed = String::new();
    edge_entry.read_to_string(&mut decompressed).unwrap();
    assert_eq!(decompressed, *edge_cuts);
}
