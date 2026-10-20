//! Linter context shared across all rules during evaluation.

use crate::netlist::Netlist;
use crate::parser::Circuit;

/// Context passed to each lint rule containing the AST, lowered netlist graph,
/// and optional hardware target metadata.
#[derive(Debug)]
pub struct LintContext<'a> {
    /// Circuit AST.
    pub circuit: &'a Circuit,
    /// Pre-computed netlist connectivity graph.
    pub netlist: &'a Netlist,
    /// Target microcontroller chip designator (e.g. "stm32g431", "rp2040", "esp32-s3").
    pub mcu: Option<&'a str>,
}

impl<'a> LintContext<'a> {
    /// Creates a new lint context with the given circuit and netlist.
    pub fn new(circuit: &'a Circuit, netlist: &'a Netlist) -> Self {
        Self {
            circuit,
            netlist,
            mcu: None,
        }
    }

    /// Sets the target MCU chip designator.
    pub fn with_mcu(mut self, mcu: Option<&'a str>) -> Self {
        self.mcu = mcu;
        self
    }
}
