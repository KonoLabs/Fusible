//! Excellon NC drill writer.

use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;

use crate::gerber::coordinates::Coord;

/// Excellon NC drill file writer.
pub struct ExcellonWriter {
    title: String,
    file_function: String,
    tools: BTreeMap<u32, f64>,
    holes: Vec<(u32, Coord, Coord)>,
    next_tool: u32,
}

impl ExcellonWriter {
    /// Creates a new drill writer with a given title and function comment.
    pub fn new(title: &str, file_function: &str) -> Self {
        Self {
            title: title.to_string(),
            file_function: file_function.to_string(),
            tools: BTreeMap::new(),
            holes: Vec::new(),
            next_tool: 1,
        }
    }

    /// Registers a tool with the given drill diameter in mm, returning its tool number.
    pub fn register_tool(&mut self, diameter_mm: f64) -> u32 {
        for (&id, &d) in &self.tools {
            if (d - diameter_mm).abs() < 1e-4 {
                return id;
            }
        }
        let id = self.next_tool;
        self.next_tool += 1;
        self.tools.insert(id, diameter_mm);
        id
    }

    /// Adds a drilled hole coordinate with the given tool ID.
    pub fn add_hole(&mut self, tool_id: u32, x: Coord, y: Coord) {
        self.holes.push((tool_id, x, y));
    }

    /// Returns true if this drill file contains any holes.
    pub fn is_empty(&self) -> bool {
        self.holes.is_empty()
    }

    /// Serializes the complete Excellon drill file.
    pub fn finish(self) -> String {
        let mut buf = String::with_capacity(4096);
        let _ = writeln!(buf, "M48");
        let _ = writeln!(buf, "; DRILL file {{{}}}", self.title);
        let _ = writeln!(buf, "; #@! TF.FileFunction,{}", self.file_function);
        let _ = writeln!(buf, "METRIC,LZ");

        for (id, dia) in &self.tools {
            let _ = writeln!(buf, "T{:02}C{:.4}", id, dia);
        }

        let _ = writeln!(buf, "%");
        let _ = writeln!(buf, "G90");
        let _ = writeln!(buf, "G05");

        let mut current_tool = None;
        for (tool, x, y) in self.holes {
            if current_tool != Some(tool) {
                let _ = writeln!(buf, "T{:02}", tool);
                current_tool = Some(tool);
            }
            let _ = writeln!(buf, "X{}Y{}", x.format_drill(), y.format_drill());
        }

        let _ = writeln!(buf, "M30");
        buf
    }
}
