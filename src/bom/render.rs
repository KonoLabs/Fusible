//! Text renderings of a report: an aligned table for the terminal, and CSV.

use super::report::{Kind, Line, Report};

const HEADER: [&str; 6] = ["PICK", "QTY", "PART", "REFS", "UNIT", "TOTAL"];

fn price(value: Option<f64>, estimated: bool) -> String {
    let mark = if estimated { "~" } else { "" };
    value.map_or_else(|| "-".into(), |v| format!("{mark}{v:.2}"))
}

fn label(line: &Line) -> String {
    match line.kind {
        Kind::Print => line.description.clone(),
        _ => line.mpn.as_ref().map_or_else(
            || line.description.clone(),
            |mpn| format!("{} ({mpn})", line.description),
        ),
    }
}

fn row(line: &Line) -> [String; 6] {
    [
        if line.picked { "x" } else { "" }.into(),
        line.quantity.to_string(),
        label(line),
        line.designators.join(" "),
        price(line.unit_price, line.estimated),
        price(line.total, line.estimated),
    ]
}

fn widths(rows: &[[String; 6]]) -> [usize; 6] {
    let mut widths = HEADER.map(str::len);
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }
    widths
}

fn pad(row: &[String; 6], widths: &[usize; 6]) -> String {
    let cells = row.iter().zip(widths).map(|(c, w)| format!("{c:<w$}"));
    cells.collect::<Vec<_>>().join("  ").trim_end().to_string()
}

/// Renders the report as an aligned table with a totals footer.
pub fn table(report: &Report) -> String {
    let mut rows = vec![HEADER.map(String::from)];
    rows.extend(report.lines.iter().map(row));
    let widths = widths(&rows);
    let mut out: Vec<String> = rows.iter().map(|r| pad(r, &widths)).collect();
    let t = &report.totals;
    let unit = &report.currency;
    out.push(String::new());
    out.push(format!("electronics  {:>10.2} {unit}", t.electronics));
    out.push(format!("printed      {:>10.2} {unit}", t.printed));
    out.push(format!("other        {:>10.2} {unit}", t.manual));
    out.push(format!("TOTAL        {:>10.2} {unit}", t.total));
    if t.unpriced > 0 {
        out.push(format!("{} picked line(s) have no price yet", t.unpriced));
    }
    if t.estimated > 0 {
        out.push(format!(
            "{} picked line(s) use an estimated price (~)",
            t.estimated
        ));
    }
    if t.estimated > 0 && report.estimate_currency != report.currency {
        out.push(format!(
            "estimates are in {}: no rate for {}",
            report.estimate_currency, report.currency
        ));
    }
    out.join("\n") + "\n"
}

fn quote(cell: &str) -> String {
    let guarded = if cell.starts_with(['=', '+', '-', '@']) {
        format!("'{cell}")
    } else {
        cell.to_string()
    };
    if guarded.contains([',', '"', '\n']) {
        format!("\"{}\"", guarded.replace('"', "\"\""))
    } else {
        guarded
    }
}

/// Renders the report as CSV, one row per line. Cells that start like a formula are prefixed with `'`.
pub fn csv(report: &Report) -> String {
    let mut out = String::from(
        "id,description,mpn,refs,quantity,unit_price,total,estimated,picked,supplier,url\n",
    );
    for line in &report.lines {
        let cells = [
            line.id.clone(),
            line.description.clone(),
            line.mpn.clone().unwrap_or_default(),
            line.designators.join(" "),
            line.quantity.to_string(),
            line.unit_price
                .map(|p| format!("{p:.4}"))
                .unwrap_or_default(),
            line.total.map(|t| format!("{t:.2}")).unwrap_or_default(),
            line.estimated.to_string(),
            line.picked.to_string(),
            line.supplier.clone().unwrap_or_default(),
            line.url.clone().unwrap_or_default(),
        ];
        out.push_str(&cells.map(|c| quote(&c)).join(","));
        out.push('\n');
    }
    out
}
