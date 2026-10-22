//! Plated and non-plated Excellon drill file builders.

use super::elements::PcbModel;
use super::geometry::Drill;
use super::holes::{hole_drill, plated_hole_geometry};
use super::math::{coord, long_axis};
use super::{invalid, GerberError};
use crate::gerber::coordinates::Coord;
use crate::gerber::excellon::ExcellonWriter;

impl PcbModel {
    /// Builds the plated drill file: plated holes and slots plus every distinct via.
    pub fn build_drill_file(&self, title: &str) -> Result<String, GerberError> {
        let mut writer = ExcellonWriter::new(title, "Plated,1,2,PTH");
        for (i, hole) in self.plated_holes.iter().enumerate() {
            plated_hole_geometry(hole)
                .and_then(|(_, drill)| drill.emit(&mut writer))
                .map_err(invalid("pcb_plated_hole", i))?;
        }
        for (i, via) in self.via_sites()?.into_iter().enumerate() {
            Drill::Round {
                d: via.hole,
                at: via.at,
            }
            .emit(&mut writer)
            .map_err(invalid("via", i))?;
        }
        Ok(writer.finish())
    }

    /// Builds the non-plated drill file from `pcb_hole` elements, or `None` when the board has none.
    pub fn build_npth_drill_file(&self, title: &str) -> Result<Option<String>, GerberError> {
        let mut writer = ExcellonWriter::new(title, "NonPlated,1,2,NPTH");
        for (i, hole) in self.holes.iter().enumerate() {
            hole_drill(hole)
                .and_then(|drill| drill.emit(&mut writer))
                .map_err(invalid("pcb_hole", i))?;
        }
        Ok((!writer.is_empty()).then(|| writer.finish()))
    }
}

impl Drill {
    /// Registers the drill's tool and adds its hit, degrading a zero-length slot to a hole.
    fn emit(self, writer: &mut ExcellonWriter) -> Result<(), String> {
        match self {
            Self::Round { d, at } => {
                let tool = writer.register_tool(d)?;
                writer.add_hole(tool, Coord::from_mm(at[0]), Coord::from_mm(at[1]));
            }
            Self::Slot { w, h, rot, at } => {
                let tool = writer.register_tool(w.min(h))?;
                let (from, to) = long_axis(w, h, rot, at);
                if coord(from) == coord(to) {
                    writer.add_hole(tool, Coord::from_mm(at[0]), Coord::from_mm(at[1]));
                } else {
                    writer.add_slot(tool, coord(from), coord(to));
                }
            }
        }
        Ok(())
    }
}
