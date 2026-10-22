//! Silicon pin-muxing, Alternate Function (AF) matrices, and hardware validation.

pub mod database;
mod designator;
pub mod model;
pub mod mux;
mod signal;

pub use database::find_target;
pub use model::{
    GpioNum, MuxFunction, PeripheralSignal, PinCapabilities, PinSpec, SiliconTarget,
    StrappingRequirement,
};
pub use mux::{validate_pin_assignments, AssignedPin, PinConflict};

#[cfg(test)]
mod tests;
