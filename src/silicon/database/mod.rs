//! Central database of microcontroller silicon targets.

/// Builds a `MuxFunction` table entry.
macro_rules! mux {
    ($sig:expr, $af:expr, $name:expr) => {
        $crate::silicon::model::MuxFunction {
            signal: $sig,
            af_number: $af,
            af_name: $name,
        }
    };
}

/// Builds a `PinSpec` for a numbered GPIO, with an
/// optional strapping requirement.
macro_rules! gpio_pin {
    ($n:expr, $name:expr, $caps:expr, $funcs:expr) => {
        $crate::silicon::model::PinSpec {
            gpio: Some($crate::silicon::model::GpioNum($n)),
            name: $name,
            capabilities: $caps,
            strapping: None,
            functions: $funcs,
        }
    };
    ($n:expr, $name:expr, $caps:expr, $funcs:expr, $strap:expr) => {
        $crate::silicon::model::PinSpec {
            gpio: Some($crate::silicon::model::GpioNum($n)),
            name: $name,
            capabilities: $caps,
            strapping: Some($strap),
            functions: $funcs,
        }
    };
}

/// Builds a `PinSpec` for a port/pin designator
/// without a GPIO number.
macro_rules! port_pin {
    ($name:expr, $caps:expr, $funcs:expr) => {
        $crate::silicon::model::PinSpec {
            gpio: None,
            name: $name,
            capabilities: $caps,
            strapping: None,
            functions: $funcs,
        }
    };
}

pub mod esp32s3;
pub mod rp2040;
pub mod stm32;

use super::model::SiliconTarget;

/// Resolves an exact microcontroller chip ID to its static silicon database
/// definition.
///
/// Matching is case-insensitive and ignores `-`, `_` and spaces, so
/// `ESP32-S3`, `esp32s3` and `esp32_s3` all resolve. STM32G431 also accepts
/// its package/flash suffixes (`stm32g431cb`, `stm32g431kb`, ...). Any other
/// chip, including other members of the same family, returns `None`.
pub fn find_target(query: &str) -> Option<&'static SiliconTarget> {
    let clean: String = query
        .trim()
        .chars()
        .filter(|c| !matches!(c, '-' | '_' | ' '))
        .map(|c| c.to_ascii_lowercase())
        .collect();

    match clean.as_str() {
        "rp2040" => Some(&rp2040::RP2040),
        "esp32s3" => Some(&esp32s3::ESP32S3),
        _ if is_stm32g431(&clean) => Some(&stm32::STM32G431),
        _ => None,
    }
}

fn is_stm32g431(clean: &str) -> bool {
    match clean.strip_prefix("stm32g431") {
        Some("") => true,
        Some(suffix) => {
            let b = suffix.as_bytes();
            b.len() == 2 && b"cgkmrv".contains(&b[0]) && b"68b".contains(&b[1])
        }
        None => false,
    }
}
