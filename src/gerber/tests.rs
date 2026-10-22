use std::io::Read;

use super::*;
use crate::gerber::apertures::Aperture;
use crate::gerber::coordinates::Coord;

fn board_with(extra: &str) -> String {
    format!(
        r#"[{{ "type": "pcb_board", "pcb_board_id": "b0", "width": 50, "height": 40, "center": {{ "x": 0, "y": 0 }} }}{extra}]"#
    )
}

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

    let id3 = dict.intern(ap1);
    assert_eq!(id3, 10);

    let mut out = String::new();
    dict.write_definitions(&mut out);
    assert!(out.contains("%ADD10C,0.200000*%"));
    assert!(out.contains("%ADD11R,1.000000X0.500000*%"));
}

#[test]
fn aperture_sizes_round_and_reject_non_positive() {
    assert_eq!(
        Aperture::circle(0.2 - 1e-12).unwrap(),
        Aperture::Circle {
            diameter_nm: 200_000
        }
    );
    assert!(Aperture::circle(0.0).is_err());
    assert!(Aperture::rectangle(1.0, -0.5).is_err());
    assert!(Aperture::obround(f64::NAN, 1.0).is_err());
}

#[test]
fn gerber_writer_structure_and_termination() {
    let mut dict = ApertureDictionary::new();
    let ap = dict.intern(Aperture::Circle {
        diameter_nm: 250_000,
    });
    let mut writer =
        GerberWriter::new("Top Copper", "Copper,L1,Top", FilePolarity::Positive, &dict);

    writer.select_aperture(ap);
    writer.move_to(Coord::from_mm(0.0), Coord::from_mm(0.0));
    writer.draw_to(Coord::from_mm(10.0), Coord::from_mm(10.0));
    writer.flash(Coord::from_mm(5.0), Coord::from_mm(5.0));

    let content = writer.finish();
    assert!(content.contains("%FSLAX46Y46*%"));
    assert!(content.contains("%MOMM*%"));
    assert!(content.contains("%LPD*%"));
    assert!(content.contains("%TF.FileFunction,Copper,L1,Top*%"));
    assert!(content.contains(&format!(
        "%TF.GenerationSoftware,Fusible,Fusible,{}*%",
        env!("CARGO_PKG_VERSION")
    )));
    assert!(content.contains("D10*"));
    assert!(content.contains("X0Y0D02*"));
    assert!(content.contains("X10000000Y10000000D01*"));
    assert!(content.contains("X5000000Y5000000D03*"));
    assert!(content.ends_with("M02*\n"));
}

#[test]
fn plot_state_is_linear_before_first_draw() {
    let writer = GerberWriter::new(
        "t",
        "Copper,L1,Top",
        FilePolarity::Positive,
        &ApertureDictionary::new(),
    );
    let content = writer.finish();
    assert!(content.lines().any(|l| l == "G01*"));
}

#[test]
fn excellon_writer_structure_and_tool_registration() {
    let mut writer = ExcellonWriter::new("MyBoard", "Plated,1,2,PTH");
    let t1 = writer.register_tool(0.8).unwrap();
    let t2 = writer.register_tool(0.3).unwrap();
    let t3 = writer.register_tool(0.8).unwrap();
    assert_eq!(t1, t3);
    assert_ne!(t1, t2);

    writer.add_hole(t1, Coord::from_mm(12.5), Coord::from_mm(25.0));
    writer.add_hole(t2, Coord::from_mm(0.0), Coord::from_mm(5.0));

    let content = writer.finish();
    assert!(content.contains("M48"));
    assert!(content.lines().any(|l| l == "METRIC"));
    assert!(!content.contains(",LZ"));
    assert!(content.contains("T01C0.8000"));
    assert!(content.contains("T02C0.3000"));
    assert!(content.contains("G05"));
    assert!(content.contains("T01\nX12.5000Y25.0000"));
    assert!(content.contains("T02\nX0.0000Y5.0000"));
    assert!(content.ends_with("M30\n"));
}

#[test]
fn excellon_rejects_bad_tools_and_sanitizes_title() {
    let mut writer = ExcellonWriter::new("evil; M30\n{x}", "Plated,1,2,PTH");
    assert!(writer.register_tool(0.0).is_err());
    assert!(writer.register_tool(-0.4).is_err());
    for i in 1..=99 {
        writer.register_tool(i as f64 * 0.01).unwrap();
    }
    assert!(writer.register_tool(5.0).is_err());
    let content = writer.finish();
    assert!(content.contains("; DRILL file {evil M30x}\n"));
}

#[test]
fn generates_complete_gerber_bundle_from_circuit_json() {
    let circuit_json = board_with(
        r#",
        { "type": "pcb_smtpad", "shape": "rect", "layer": "top", "x": 2.5, "y": 1.0, "width": 1.2, "height": 0.6 },
        { "type": "pcb_plated_hole", "shape": "circle", "x": -5.0, "y": 0.0, "hole_diameter": 0.8, "outer_diameter": 1.6 },
        { "type": "pcb_trace", "route": [
            { "route_type": "wire", "x": 0.0, "y": 0.0, "width": 0.2, "layer": "top" },
            { "route_type": "wire", "x": 2.5, "y": 1.0, "width": 0.2, "layer": "top" }
        ] }"#,
    );

    let result = generate_gerber_files(&circuit_json, "TestBoard");
    let files = result.as_ref().expect("valid circuit");
    for name in [
        "Edge_Cuts.gbr",
        "F_Cu.gbr",
        "B_Cu.gbr",
        "F_Mask.gbr",
        "B_Mask.gbr",
        "drill_pth.drl",
    ] {
        assert!(files.contains_key(name), "{name}");
    }
    assert!(!files.contains_key("drill_npth.drl"));

    let f_cu = &files["F_Cu.gbr"];
    assert!(f_cu.contains("Copper,L1,Top"));
    assert!(f_cu.contains("D03*"));
    assert!(f_cu.contains("D01*"));

    let zip_bytes = create_gerber_zip(result.as_ref().expect("valid board")).expect("zip creation");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).expect("valid zip");
    let mut decompressed = String::new();
    archive
        .by_name("Edge_Cuts.gbr")
        .expect("entry exists")
        .read_to_string(&mut decompressed)
        .unwrap();
    assert_eq!(decompressed, files["Edge_Cuts.gbr"]);
}

#[test]
fn edge_cuts_close_an_open_outline() {
    let json = r#"[{ "type": "pcb_board", "outline": [
        { "x": 0, "y": 0 }, { "x": 10, "y": 0 }, { "x": 10, "y": 5 }
    ] }]"#;
    let edge = PcbModel::try_parse(json)
        .unwrap()
        .build_edge_cuts()
        .unwrap();
    let ops: Vec<&str> = edge.lines().filter(|l| l.starts_with('X')).collect();
    assert_eq!(ops.first(), Some(&"X0Y0D02*"));
    assert_eq!(ops.last(), Some(&"X0Y0D01*"));
    assert_eq!(ops.len(), 4);

    let rect = PcbModel::try_parse(&board_with(""))
        .unwrap()
        .build_edge_cuts()
        .unwrap();
    assert_eq!(rect.lines().filter(|l| l.ends_with("D01*")).count(), 4);
}

#[test]
fn trace_vias_get_copper_on_both_layers_without_double_drilling() {
    let json = board_with(
        r#",
        { "type": "pcb_via", "x": 1, "y": 1, "outer_diameter": 0.6, "hole_diameter": 0.3, "layers": ["top", "bottom"] },
        { "type": "pcb_trace", "route": [
            { "route_type": "wire", "x": 0, "y": 1, "width": 0.2, "layer": "top" },
            { "route_type": "via", "x": 1, "y": 1, "from_layer": "top", "to_layer": "bottom" },
            { "route_type": "wire", "x": 3, "y": 1, "width": 0.2, "layer": "bottom" },
            { "route_type": "via", "x": 3, "y": 1, "from_layer": "bottom", "to_layer": "top", "outer_diameter": 0.7, "hole_diameter": 0.35 },
            { "route_type": "wire", "x": 4, "y": 1, "width": 0.2, "layer": "top" }
        ] }"#,
    );
    let files = generate_gerber_files(&json, "t").unwrap();
    for layer in ["F_Cu.gbr", "B_Cu.gbr"] {
        assert!(files[layer].contains("C,0.700000*%"), "{layer}");
        assert!(files[layer].contains("X3000000Y1000000D03*"), "{layer}");
        assert_eq!(files[layer].matches("X1000000Y1000000D03*").count(), 1);
    }
    let drill = &files["drill_pth.drl"];
    assert_eq!(drill.matches("X1.0000Y1.0000").count(), 1);
    assert_eq!(drill.matches("X3.0000Y1.0000").count(), 1);
}

#[test]
fn trace_via_without_diameters_fails_closed() {
    let json = board_with(
        r#",
        { "type": "pcb_trace", "route": [
            { "route_type": "wire", "x": 0, "y": 1, "width": 0.2, "layer": "top" },
            { "route_type": "via", "x": 1, "y": 1 },
            { "route_type": "wire", "x": 2, "y": 1, "width": 0.2, "layer": "bottom" }
        ] }"#,
    );
    assert!(generate_gerber_files(&json, "t").is_err());
}

#[test]
fn solder_mask_is_negative_and_copper_positive() {
    let model = PcbModel::try_parse(&board_with("")).unwrap();
    assert!(model
        .build_mask_layer(true)
        .unwrap()
        .contains("%TF.FilePolarity,Negative*%"));
    assert!(model
        .build_copper_layer(true)
        .unwrap()
        .contains("%TF.FilePolarity,Positive*%"));
}

#[test]
fn rotated_pads_honor_ccw_rotation() {
    let json = board_with(
        r#",
        { "type": "pcb_smtpad", "shape": "rotated_rect", "layer": "top", "x": 0, "y": 0, "width": 2, "height": 1, "ccw_rotation": 90 },
        { "type": "pcb_smtpad", "shape": "rotated_rect", "layer": "top", "x": 5, "y": 0, "width": 2, "height": 1, "ccw_rotation": 45 },
        { "type": "pcb_smtpad", "shape": "rotated_pill", "layer": "top", "x": 9, "y": 0, "width": 2, "height": 1, "radius": 0.5, "ccw_rotation": 30 }"#,
    );
    let cu = PcbModel::try_parse(&json)
        .unwrap()
        .build_copper_layer(true)
        .unwrap();
    assert!(cu.contains("R,1.000000X2.000000*%"));
    assert!(cu.contains("G36*"));
    assert!(cu.contains("G37*"));
    assert!(cu.contains("C,1.000000*%"));
    assert!(cu.contains("X8566987Y-250000D02*"));
    assert!(cu.contains("X9433013Y250000D01*"));
}

#[test]
fn plated_hole_shapes_use_their_own_pad_and_drill_fields() {
    let json = board_with(
        r#",
        { "type": "pcb_plated_hole", "shape": "pill", "x": 0, "y": 0, "outer_width": 3, "outer_height": 1.5, "hole_width": 2, "hole_height": 0.8 },
        { "type": "pcb_plated_hole", "shape": "circular_hole_with_rect_pad", "x": 5, "y": 0, "hole_diameter": 1, "rect_pad_width": 2, "rect_pad_height": 1.8 }"#,
    );
    let files = generate_gerber_files(&json, "t").unwrap();
    assert!(files["F_Cu.gbr"].contains("O,3.000000X1.500000*%"));
    assert!(files["F_Cu.gbr"].contains("R,2.000000X1.800000*%"));
    let drill = &files["drill_pth.drl"];
    assert!(drill.contains("C0.8000"));
    assert!(drill.contains("G00X-0.6000Y0.0000\nM15\nG01X0.6000Y0.0000\nM16\nG05"));
}

#[test]
fn unknown_shapes_fail_instead_of_defaulting() {
    for extra in [
        r#", { "type": "pcb_smtpad", "shape": "polygon", "layer": "top", "x": 0, "y": 0, "points": [] }"#,
        r#", { "type": "pcb_smtpad", "layer": "top", "x": 0, "y": 0, "width": 1, "height": 1 }"#,
        r#", { "type": "pcb_smtpad", "shape": "rect", "layer": "top", "x": 0, "y": 0 }"#,
        r#", { "type": "pcb_plated_hole", "shape": "hole_with_polygon_pad", "x": 0, "y": 0 }"#,
        r#", { "type": "pcb_hole", "hole_shape": "square", "x": 0, "y": 0, "hole_diameter": 1 }"#,
    ] {
        assert!(
            generate_gerber_files(&board_with(extra), "t").is_err(),
            "{extra}"
        );
    }
}

#[test]
fn strict_parse_rejects_bad_input() {
    assert!(matches!(
        PcbModel::try_parse("not json"),
        Err(GerberError::InvalidJson(_))
    ));
    assert!(matches!(
        PcbModel::try_parse("[]"),
        Err(GerberError::NoBoard)
    ));
    assert!(matches!(
        PcbModel::try_parse(&board_with(
            r#", { "type": "pcb_smtpad", "x": "oops", "y": 0 }"#
        )),
        Err(GerberError::Malformed { index: 1, .. })
    ));
    assert!(generate_gerber_files(r#"[{ "type": "pcb_board" }]"#, "t").is_err());
    let no_layer = board_with(
        r#", { "type": "pcb_trace", "route": [
            { "route_type": "wire", "x": 0, "y": 0, "width": 0.2 },
            { "route_type": "wire", "x": 1, "y": 0, "width": 0.2 }
        ] }"#,
    );
    assert!(generate_gerber_files(&no_layer, "t").is_err());
    assert!(generate_gerber_files("[]", "t").is_err());
    assert_eq!(PcbModel::parse("not json").boards.len(), 0);
}

#[test]
fn mounting_holes_go_to_npth_drill_file() {
    let json = board_with(
        r#", { "type": "pcb_hole", "hole_shape": "circle", "x": 3, "y": 4, "hole_diameter": 3.2 }"#,
    );
    let files = generate_gerber_files(&json, "t").unwrap();
    let npth = &files["drill_npth.drl"];
    assert!(npth.contains("TF.FileFunction,NonPlated,1,2,NPTH"));
    assert!(npth.contains("T01C3.2000\n"));
    assert!(npth.contains("X3.0000Y4.0000"));
    assert!(!files["drill_pth.drl"].contains("X3.0000Y4.0000"));
}

#[test]
fn zero_width_silkscreen_is_skipped_but_negative_fails() {
    let path = |w: &str| {
        board_with(&format!(
            r#", {{ "type": "pcb_silkscreen_path", "layer": "top", "stroke_width": {w}, "route": [{{ "x": 0, "y": 0 }}, {{ "x": 1, "y": 0 }}] }}"#
        ))
    };
    let silk = generate_gerber_files(&path("0"), "t").unwrap();
    assert!(!silk["F_Silk.gbr"].contains("D01*"));
    assert!(generate_gerber_files(&path("-0.1"), "t").is_err());
}

#[test]
fn test_mounting_holes_get_their_own_npth_drill_file() {
    let with_hole = r#"[
        { "type": "pcb_board", "width": 20.0, "height": 20.0 },
        { "type": "pcb_hole", "hole_shape": "circle", "x": 5.0, "y": -5.0, "hole_diameter": 3.2 }
    ]"#;
    let files = generate_gerber_files(with_hole, "TestBoard").unwrap();
    let npth = &files["drill_npth.drl"];
    assert!(npth.contains("NonPlated,1,2,NPTH"));
    assert!(npth.contains("C3.2000"));
    assert!(npth.contains("X5.0000Y-5.0000"));
    assert!(!files["drill_pth.drl"].contains("C3.2000"));

    let without = generate_gerber_files(
        r#"[{ "type": "pcb_board", "width": 20.0, "height": 20.0 }]"#,
        "TestBoard",
    )
    .unwrap();
    assert!(!without.contains_key("drill_npth.drl"));
}
