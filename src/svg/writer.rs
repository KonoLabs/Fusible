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
    buf: String,
}

impl SvgWriter {
    pub fn new() -> Self {
        Self {
            buf: String::with_capacity(4096),
        }
    }

    pub fn start_svg(
        &mut self,
        width: f64,
        height: f64,
        min_x: f64,
        min_y: f64,
        view_w: f64,
        view_h: f64,
    ) {
        let _ = write!(
            self.buf,
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{min_x:.3} {min_y:.3} {view_w:.3} {view_h:.3}" width="{width:.1}mm" height="{height:.1}mm">"#
        );
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

    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, fill: &str) {
        let fill = escape_xml(fill);
        let _ = writeln!(
            self.buf,
            r#"    <rect x="{x:.3}" y="{y:.3}" width="{w:.3}" height="{h:.3}" fill="{fill}" />"#
        );
    }

    pub fn rounded_rect(
        &mut self,
        bounds: [f64; 4],
        rx: f64,
        fill: &str,
        stroke: Option<(&str, f64)>,
    ) {
        let fill = escape_xml(fill);
        let [x, y, w, h] = bounds;
        let _ = write!(
            self.buf,
            r#"    <rect x="{x:.3}" y="{y:.3}" width="{w:.3}" height="{h:.3}" rx="{rx:.3}" fill="{fill}""#
        );
        if let Some((s_color, s_width)) = stroke {
            let s_color = escape_xml(s_color);
            let _ = write!(
                self.buf,
                r#" stroke="{s_color}" stroke-width="{s_width:.3}""#
            );
        }
        self.buf.push_str(" />\n");
    }

    pub fn circle(&mut self, cx: f64, cy: f64, r: f64, fill: &str, stroke: Option<(&str, f64)>) {
        let fill = escape_xml(fill);
        let _ = write!(
            self.buf,
            r#"    <circle cx="{cx:.3}" cy="{cy:.3}" r="{r:.3}" fill="{fill}""#
        );
        if let Some((s_color, s_width)) = stroke {
            let s_color = escape_xml(s_color);
            let _ = write!(
                self.buf,
                r#" stroke="{s_color}" stroke-width="{s_width:.3}""#
            );
        }
        self.buf.push_str(" />\n");
    }

    pub fn path(&mut self, d: &str, fill: &str, stroke: Option<(&str, f64)>) {
        let d = escape_xml(d);
        let fill = escape_xml(fill);
        let _ = write!(self.buf, r#"    <path d="{d}" fill="{fill}""#);
        if let Some((s_color, s_width)) = stroke {
            let s_color = escape_xml(s_color);
            let _ = write!(
                self.buf,
                r#" stroke="{s_color}" stroke-width="{s_width:.3}" stroke-linecap="round" stroke-linejoin="round""#
            );
        }
        self.buf.push_str(" />\n");
    }

    pub fn text(&mut self, x: f64, y: f64, text: &str, font_size: f64, fill: &str, anchor: &str) {
        let text = escape_xml(text);
        let fill = escape_xml(fill);
        let anchor = escape_xml(anchor);
        let _ = write!(
            self.buf,
            r#"    <text x="{x:.3}" y="{y:.3}" font-family="sans-serif, monospace" font-size="{font_size:.3}" fill="{fill}" text-anchor="{anchor}">{text}</text>"#
        );
        self.buf.push('\n');
    }

    pub fn finish(self) -> String {
        self.buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_and_attributes_are_escaped() {
        let mut w = SvgWriter::new();
        w.start_group("g\"><script>", None);
        w.text(
            0.0,
            0.0,
            "<img src=x onerror='a&b'>",
            1.0,
            "red\" onload=\"x",
            "middle",
        );
        w.path("M0 0", "none", Some(("#000\"", 1.0)));
        let out = w.finish();
        assert!(!out.contains("<script>"));
        assert!(!out.contains("<img"));
        assert!(out.contains("&lt;img src=x onerror=&#39;a&amp;b&#39;&gt;"));
        assert!(out.contains(r#"fill="red&quot; onload=&quot;x""#));
        assert!(out.contains(r#"id="g&quot;&gt;&lt;script&gt;""#));
        assert!(out.contains(r##"stroke="#000&quot;""##));
    }

    #[test]
    fn plain_strings_are_borrowed() {
        assert!(matches!(escape_xml("R1"), Cow::Borrowed("R1")));
    }
}
