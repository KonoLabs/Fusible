//! Low-level SVG XML string builder.

use std::fmt::Write;

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
        let [x, y, w, h] = bounds;
        let _ = write!(
            self.buf,
            r#"    <rect x="{x:.3}" y="{y:.3}" width="{w:.3}" height="{h:.3}" rx="{rx:.3}" fill="{fill}""#
        );
        if let Some((s_color, s_width)) = stroke {
            let _ = write!(
                self.buf,
                r#" stroke="{s_color}" stroke-width="{s_width:.3}""#
            );
        }
        self.buf.push_str(" />\n");
    }

    pub fn circle(&mut self, cx: f64, cy: f64, r: f64, fill: &str, stroke: Option<(&str, f64)>) {
        let _ = write!(
            self.buf,
            r#"    <circle cx="{cx:.3}" cy="{cy:.3}" r="{r:.3}" fill="{fill}""#
        );
        if let Some((s_color, s_width)) = stroke {
            let _ = write!(
                self.buf,
                r#" stroke="{s_color}" stroke-width="{s_width:.3}""#
            );
        }
        self.buf.push_str(" />\n");
    }

    pub fn path(&mut self, d: &str, fill: &str, stroke: Option<(&str, f64)>) {
        let _ = write!(self.buf, r#"    <path d="{d}" fill="{fill}""#);
        if let Some((s_color, s_width)) = stroke {
            let _ = write!(
                self.buf,
                r#" stroke="{s_color}" stroke-width="{s_width:.3}" stroke-linecap="round" stroke-linejoin="round""#
            );
        }
        self.buf.push_str(" />\n");
    }

    pub fn text(&mut self, x: f64, y: f64, text: &str, font_size: f64, fill: &str, anchor: &str) {
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
