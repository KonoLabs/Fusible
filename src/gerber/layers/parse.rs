//! Lenient and strict Circuit JSON parsing into a [`PcbModel`].

use serde::de::DeserializeOwned;
use serde_json::Value;

use super::elements::{PcbModel, PcbSmtPadElm};
use super::GerberError;

impl PcbModel {
    /// Leniently parses Circuit JSON, skipping elements that fail to deserialize.
    /// For previews only, never for fabrication.
    pub fn parse(circuit_json: &str) -> Self {
        let mut model = Self::default();
        let elms: Vec<Value> = serde_json::from_str(circuit_json).unwrap_or_default();
        for (index, val) in elms.into_iter().enumerate() {
            let _ = model.push(index, val);
        }
        model
    }

    /// Strictly parses Circuit JSON for fabrication: invalid JSON, malformed PCB elements,
    /// or a missing board are errors.
    pub fn try_parse(circuit_json: &str) -> Result<Self, GerberError> {
        let elms: Vec<Value> = serde_json::from_str(circuit_json)
            .map_err(|e| GerberError::InvalidJson(e.to_string()))?;
        let mut model = Self::default();
        for (index, val) in elms.into_iter().enumerate() {
            model.push(index, val)?;
        }
        if model.boards.is_empty() {
            return Err(GerberError::NoBoard);
        }
        Ok(model)
    }

    fn push(&mut self, index: usize, val: Value) -> Result<(), GerberError> {
        let Some(typ) = val.get("type").and_then(Value::as_str).map(str::to_owned) else {
            return Ok(());
        };
        match typ.as_str() {
            "pcb_board" => self.boards.push(decode(&typ, index, val)?),
            "pcb_smtpad" => self
                .smt_pads
                .push(positioned_pad(index, decode(&typ, index, val)?)?),
            "pcb_trace" => self.traces.push(decode(&typ, index, val)?),
            "pcb_plated_hole" => self.plated_holes.push(decode(&typ, index, val)?),
            "pcb_via" => self.vias.push(decode(&typ, index, val)?),
            "pcb_silkscreen_path" => self.silkscreen_paths.push(decode(&typ, index, val)?),
            "pcb_silkscreen_text" => self.silkscreen_texts.push(decode(&typ, index, val)?),
            "pcb_component" => self.components.push(decode(&typ, index, val)?),
            "pcb_hole" => self.holes.push(decode(&typ, index, val)?),
            _ => {}
        }
        Ok(())
    }
}

fn decode<T: DeserializeOwned>(kind: &str, index: usize, val: Value) -> Result<T, GerberError> {
    serde_json::from_value(val).map_err(|e| GerberError::Malformed {
        kind: kind.to_string(),
        index,
        reason: e.to_string(),
    })
}

/// Rejects a non-polygon pad without a finite `x` and `y`: only polygon pads
/// may omit their center, since their `points` carry the position.
fn positioned_pad(index: usize, pad: PcbSmtPadElm) -> Result<PcbSmtPadElm, GerberError> {
    if pad.shape.as_deref() == Some("polygon") || (pad.x.is_finite() && pad.y.is_finite()) {
        return Ok(pad);
    }
    Err(GerberError::Malformed {
        kind: "pcb_smtpad".to_string(),
        index,
        reason: "missing or non-finite x/y".to_string(),
    })
}
