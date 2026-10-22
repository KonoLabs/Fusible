//! Unit tests for the coordinate, aperture, Gerber and Excellon writers.

use super::super::*;
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
