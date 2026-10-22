//! Low-level SVG XML string builder.

use std::borrow::Cow;
use std::fmt::Write;

/// Escapes `& < > " '` so untrusted strings are safe in SVG text and attribute values.
pub fn escape_xml(input: &str) -> Cow<'_, str> {
    if !input.contains(['&', '<', '>', '"', '\'']) {
        return Cow::Borrowed(input);
    }
    let mut out = String::with_capacity(input.len() + 16);
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    Cow::Owned(out)
}

/// A lightweight, allocation-efficient SVG vector canvas.
#[derive(Debug, Default)]
pub struct SvgWriter {
    pub(super) buf: String,
}

impl SvgWriter {
    pub fn new() -> Self {
        Self {
            buf: String::with_capacity(4096),
        }
    }

    /// Opens the `<svg>` root with a physical `[width, height]` in mm and a
    /// `[min_x, min_y, view_w, view_h]` view box.
    pub fn start_svg(&mut self, size: [f64; 2], view_box: [f64; 4]) {
        let [width, height] = size;
        let [min_x, min_y, view_w, view_h] = view_box;
        self.buf
            .push_str(r#"<svg xmlns="http://www.w3.org/2000/svg""#);
        let _ = write!(
            self.buf,
            r#" viewBox="{min_x:.3} {min_y:.3} {view_w:.3} {view_h:.3}""#
        );
        let _ = write!(self.buf, r#" width="{width:.1}mm" height="{height:.1}mm">"#);
        self.buf.push('\n');
    }

    pub fn end_svg(&mut self) {
        self.buf.push_str("</svg>\n");
    }

    pub fn start_group(&mut self, id: &str, opacity: Option<f64>) {
        let id = escape_xml(id);
        let _ = write!(self.buf, r#"  <g id="{id}""#);
        if let Some(op) = opacity {
            let _ = write!(self.buf, r#" opacity="{op:.2}""#);
        }
        self.buf.push_str(">\n");
    }

    pub fn end_group(&mut self) {
        self.buf.push_str("  </g>\n");
    }

    pub fn finish(self) -> String {
        self.buf
    }
}
