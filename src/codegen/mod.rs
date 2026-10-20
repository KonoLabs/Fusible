//! Code generation for firmware HAL and pin bindings.

pub mod pins;

#[cfg(test)]
mod tests;

pub use pins::{extract_mcu_pins, generate_pins_rs, McuInfo, PinMapping};
