use std::path::Path;

use super::live_tests::scratch;
use super::locate::board_file;

#[test]
fn the_manifest_path_wins_over_the_file_next_to_the_entry() {
    let dir = scratch("locate_config");
    std::fs::write(dir.join("other.kicad_pcb"), "").unwrap();
    let configured = Path::new("hardware/board.kicad_pcb");
    let found = board_file(Some(configured), &dir.join("index.tsx"));
    assert_eq!(found.as_deref(), Some(configured));
}

#[test]
fn a_single_board_file_is_found_beside_the_entry_or_in_its_kicad_folder() {
    let dir = scratch("locate_single");
    let entry = dir.join("index.tsx");
    assert_eq!(board_file(None, &entry), None);
    std::fs::create_dir_all(dir.join("kicad")).unwrap();
    std::fs::write(dir.join("kicad/a.kicad_pcb"), "").unwrap();
    assert_eq!(
        board_file(None, &entry),
        Some(dir.join("kicad/a.kicad_pcb"))
    );
    std::fs::write(dir.join("b.kicad_pcb"), "").unwrap();
    assert_eq!(board_file(None, &entry), Some(dir.join("b.kicad_pcb")));
}

#[test]
fn several_board_files_beside_the_entry_are_ambiguous() {
    let dir = scratch("locate_many");
    std::fs::write(dir.join("a.kicad_pcb"), "").unwrap();
    std::fs::write(dir.join("b.kicad_pcb"), "").unwrap();
    assert_eq!(board_file(None, &dir.join("index.tsx")), None);
}
