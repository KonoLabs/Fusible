//! Central database of microcontroller silicon targets.

pub mod esp32s3;
pub mod rp2040;
pub mod stm32;

use super::model::SiliconTarget;

/// Resolves a microcontroller target name (case-insensitive, substring-aware)
/// to its static silicon database definition.
pub fn find_target(query: &str) -> Option<&'static SiliconTarget> {
    let clean = query.trim().to_ascii_lowercase();

    if clean.contains("rp2040") || clean.contains("pico") {
        return Some(&rp2040::RP2040);
    }

    if clean.contains("stm32") {
        return Some(&stm32::STM32G431);
    }

    if clean.contains("esp32") || clean.contains("esp32-s3") || clean.contains("esp32s3") {
        return Some(&esp32s3::ESP32S3);
    }

    None
}
