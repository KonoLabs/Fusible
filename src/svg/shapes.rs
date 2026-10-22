//! Shape and text primitives drawn onto an [`SvgWriter`].

use std::fmt::Write;

use super::writer::{escape_xml, SvgWriter};

/// Font size, fill colour and `text-anchor` of an SVG text element.
#[derive(Debug, Clone, Copy)]
pub struct TextStyle<'a> {
    pub font_size: f64,
    pub fill: &'a str,
    pub anchor: &'a str,
}

const ROUND_CAPS: &str = r#" stroke-linecap="round" stroke-linejoin="round""#;

impl SvgWriter {
    /// Draws a `[x, y, w, h]` rectangle.
    pub fn rect(&mut self, bounds: [f64; 4], fill: &str) {
        let fill = escape_xml(fill);
        let [x, y, w, h] = bounds;
        let _ = writeln!(
            self.buf,
            r#"    <rect x="{x:.3}" y="{y:.3}" width="{w:.3}" height="{h:.3}" fill="{fill}" />"#
        );
    }

    /// Draws a `[x, y, w, h]` rectangle with corner radius `rx` and an optional stroke.
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
        self.close_with_stroke(stroke, "");
    }

    /// Draws a circle of radius `r` centred on `[cx, cy]` with an optional stroke.
    pub fn circle(&mut self, center: [f64; 2], r: f64, fill: &str, stroke: Option<(&str, f64)>) {
        let fill = escape_xml(fill);
        let [cx, cy] = center;
        let _ = write!(
            self.buf,
            r#"    <circle cx="{cx:.3}" cy="{cy:.3}" r="{r:.3}" fill="{fill}""#
        );
        self.close_with_stroke(stroke, "");
    }

    /// Draws a path with round caps and joins on its optional stroke.
    pub fn path(&mut self, d: &str, fill: &str, stroke: Option<(&str, f64)>) {
        let d = escape_xml(d);
        let fill = escape_xml(fill);
        let _ = write!(self.buf, r#"    <path d="{d}" fill="{fill}""#);
        self.close_with_stroke(stroke, ROUND_CAPS);
    }

    /// Draws `text` anchored at `[x, y]`.
    pub fn text(&mut self, at: [f64; 2], text: &str, style: TextStyle<'_>) {
        let text = escape_xml(text);
        let fill = escape_xml(style.fill);
        let anchor = escape_xml(style.anchor);
        let [x, y] = at;
        let font_size = style.font_size;
        let _ = write!(
            self.buf,
            r#"    <text x="{x:.3}" y="{y:.3}" font-family="sans-serif, monospace""#
        );
        let _ = write!(
            self.buf,
            r#" font-size="{font_size:.3}" fill="{fill}" text-anchor="{anchor}">{text}</text>"#
        );
        self.buf.push('\n');
    }

    /// Appends the optional stroke attributes plus `extra`, then closes the element.
    fn close_with_stroke(&mut self, stroke: Option<(&str, f64)>, extra: &str) {
        if let Some((s_color, s_width)) = stroke {
            let s_color = escape_xml(s_color);
            let _ = write!(
                self.buf,
                r#" stroke="{s_color}" stroke-width="{s_width:.3}"{extra}"#
            );
        }
        self.buf.push_str(" />\n");
    }
}
