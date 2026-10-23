//! Code generation for firmware HAL and pin bindings.

mod groups;
mod naming;
pub mod pins;
mod render;
mod select;

#[cfg(test)]
mod tests;

pub(crate) use groups::{categorize_pins, field_names};
pub use pins::{extract_mcu_pins, generate_pins_rs, McuInfo, PinMapping};
