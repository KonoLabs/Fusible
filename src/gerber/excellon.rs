//! Excellon (XNC) NC drill writer.

use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;

use crate::gerber::coordinates::Coord;

const MAX_TOOL: u32 = 99;
const MAX_TITLE_CHARS: usize = 200;

#[derive(Debug, Clone, Copy)]
enum Hit {
    Drill(Coord, Coord),
    Slot([Coord; 2], [Coord; 2]),
}

/// Excellon NC drill file writer following the XNC subset (Ucamco XNC spec 2021.11).
pub struct ExcellonWriter {
    title: String,
    file_function: String,
    tools: BTreeMap<u32, Coord>,
    hits: BTreeMap<u32, Vec<Hit>>,
}

impl ExcellonWriter {
    /// Creates a new drill writer with a given title and function comment.
    pub fn new(title: &str, file_function: &str) -> Self {
        Self {
            title: sanitize_comment(title),
            file_function: file_function.to_string(),
            tools: BTreeMap::new(),
            hits: BTreeMap::new(),
        }
    }

    /// Registers a tool with the given finished diameter in mm and returns its number (01-99).
    pub fn register_tool(&mut self, diameter_mm: f64) -> Result<u32, String> {
        let diameter = Coord::from_mm((diameter_mm * 10_000.0).round() / 10_000.0);
        if !diameter_mm.is_finite() || diameter.0 <= 0 {
            return Err(format!("drill diameter {diameter_mm} mm must be positive"));
        }
        if let Some((&id, _)) = self.tools.iter().find(|(_, &d)| d == diameter) {
            return Ok(id);
        }
        let id = self.tools.len() as u32 + 1;
        if id > MAX_TOOL {
            return Err(format!(
                "drill file needs more than {MAX_TOOL} distinct tool diameters"
            ));
        }
        self.tools.insert(id, diameter);
        Ok(id)
    }

    /// Adds a drilled hole coordinate with the given tool ID.
    pub fn add_hole(&mut self, tool_id: u32, x: Coord, y: Coord) {
        self.hits.entry(tool_id).or_default().push(Hit::Drill(x, y));
    }

    /// Adds a straight routed slot between two points with the given tool ID.
    pub fn add_slot(&mut self, tool_id: u32, from: [Coord; 2], to: [Coord; 2]) {
        self.hits
            .entry(tool_id)
            .or_default()
            .push(Hit::Slot(from, to));
    }

    /// Returns true if this drill file contains any holes.
    pub fn is_empty(&self) -> bool {
        self.hits.is_empty()
    }

    /// Serializes the complete Excellon drill file.
    pub fn finish(self) -> String {
        let mut buf = String::with_capacity(4096);
        self.write_header(&mut buf);

        for (tool, hits) in &self.hits {
            let _ = writeln!(buf, "T{:02}", tool);
            for hit in hits {
                match *hit {
                    Hit::Drill(x, y) => {
                        let _ = writeln!(buf, "X{}Y{}", x.format_drill(), y.format_drill());
                    }
                    Hit::Slot([x1, y1], [x2, y2]) => {
                        let _ = writeln!(buf, "G00X{}Y{}", x1.format_drill(), y1.format_drill());
                        let _ = writeln!(buf, "M15");
                        let _ = writeln!(buf, "G01X{}Y{}", x2.format_drill(), y2.format_drill());
                        let _ = writeln!(buf, "M16");
                        let _ = writeln!(buf, "G05");
                    }
                }
            }
        }

        let _ = writeln!(buf, "M30");
        buf
    }

    /// Writes the M48 header: title, attributes, units and the tool table.
    fn write_header(&self, buf: &mut String) {
        let _ = writeln!(buf, "M48");
        let _ = writeln!(buf, "; DRILL file {{{}}}", self.title);
        let _ = writeln!(
            buf,
            "; #@! TF.GenerationSoftware,Fusible,Fusible,{}",
            env!("CARGO_PKG_VERSION")
        );
        let _ = writeln!(buf, "; #@! TF.FileFunction,{}", self.file_function);
        let _ = writeln!(buf, "METRIC");

        for (id, dia) in &self.tools {
            let _ = writeln!(buf, "T{:02}C{}", id, dia.format_drill());
        }

        let _ = writeln!(buf, "%");
        let _ = writeln!(buf, "G05");
    }
}

/// Strips characters that would end or corrupt an XNC comment and caps its length.
fn sanitize_comment(text: &str) -> String {
    text.chars()
        .filter(|c| *c != ';' && *c != '{' && *c != '}' && !c.is_control())
        .take(MAX_TITLE_CHARS)
        .collect()
}
