//! Unit tests for the SVG writer and the PCB and schematic renderers.

use std::borrow::Cow;

use super::shapes::TextStyle;
use super::writer::{escape_xml, SvgWriter};
use super::*;

#[test]
fn renders_valid_pcb_svg() {
    let json = r#"[
        { "type": "pcb_board", "width": 50.0, "height": 30.0, "center": { "x": 0.0, "y": 0.0 } },
        { "type": "pcb_smtpad", "x": 10.0, "y": 5.0, "width": 1.5, "height": 2.0, "layer": "top" },
        { "type": "pcb_via", "x": -5.0, "y": -5.0, "outer_diameter": 0.8, "hole_diameter": 0.4 }
    ]"#;

    let svg = render_pcb_svg(json);
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>\n"));
    assert!(svg.contains("substrate"));
    assert!(svg.contains("pads"));
    assert!(svg.contains("holes_and_vias"));
}

#[test]
fn pcb_svg_draws_board_from_outline_points() {
    let svg = render_pcb_svg(
        r#"[{ "type": "pcb_board", "width": 50.0, "height": 30.0, "center": { "x": 0.0, "y": 0.0 } }]"#,
    );
    assert!(svg.contains("M -25.000 15.000 L 25.000 15.000 L 25.000 -15.000"));
}

#[test]
fn empty_schematic_is_still_a_valid_svg() {
    let circuit = crate::parser::parse("[]", Path::new("empty.json")).unwrap();
    let netlist = Netlist::from_circuit(&circuit);
    let svg = render_schematic_svg(&circuit, &netlist);
    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>\n"));
}

#[test]
fn renders_valid_schematic_svg() {
    let json = r#"[
        { "type": "source_component", "component_id": "U1", "name": "MCU", "kind": "chip" },
        { "type": "source_component", "component_id": "R1", "name": "R1", "kind": "resistor" },
        { "type": "source_net", "net_id": "n1", "name": "SIG" },
        { "type": "source_port", "port_id": "U1.1", "source_component_id": "U1", "name": "IO1", "net_id": "n1" },
        { "type": "source_port", "port_id": "R1.1", "source_component_id": "R1", "name": "1", "net_id": "n1" }
    ]"#;

    let circuit = crate::parser::parse(json, Path::new("test.json")).unwrap();
    let netlist = Netlist::from_circuit(&circuit);
    let svg = render_schematic_svg(&circuit, &netlist);

    assert!(svg.starts_with("<svg"));
    assert!(svg.ends_with("</svg>\n"));
    assert!(svg.contains("components"));
    assert!(svg.contains("nets"));
    assert!(svg.contains("MCU"));
}

#[test]
fn text_and_attributes_are_escaped() {
    let mut w = SvgWriter::new();
    w.start_group("g\"><script>", None);
    w.text(
        [0.0, 0.0],
        "<img src=x onerror='a&b'>",
        TextStyle {
            font_size: 1.0,
            fill: "red\" onload=\"x",
            anchor: "middle",
        },
    );
    w.path("M0 0", "none", Some(("#000\"", 1.0)));
    let out = w.finish();
    assert!(!out.contains("<script>"));
    assert!(!out.contains("<img"));
    assert!(out.contains("&lt;img src=x onerror=&#39;a&amp;b&#39;&gt;"));
    assert!(out.contains(r#"fill="red&quot; onload=&quot;x""#));
    assert!(out.contains(r#"id="g&quot;&gt;&lt;script&gt;""#));
    assert!(out.contains(r##"stroke="#000&quot;""##));
}

#[test]
fn plain_strings_are_borrowed() {
    assert!(matches!(escape_xml("R1"), Cow::Borrowed("R1")));
}
