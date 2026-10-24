use super::*;

#[test]
fn a_board_may_name_its_kicad_file() {
    let yaml = "boards:\n  main: { entry: a.tsx, kicad: hardware/b.kicad_pcb }\n";
    let cfg = FusibleConfig::from_str(yaml).unwrap();
    let kicad = cfg.boards["main"].kicad.as_deref();
    assert_eq!(kicad, Some(std::path::Path::new("hardware/b.kicad_pcb")));
}
