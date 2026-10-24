//! Merges detected parts, manual parts, and printed bodies into priced lines with totals.

use std::collections::BTreeMap;

use serde::Serialize;

use super::estimate::{unit_price, Body};
use super::file::{BomFile, PartEntry, PrintEntry};
use super::Part;

const DEFAULT_CURRENCY: &str = "EUR";

/// Where a line comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Component,
    Manual,
    Print,
}

/// One row of the bill of materials.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Line {
    pub id: String,
    pub kind: Kind,
    pub description: String,
    pub value: Option<String>,
    pub mpn: Option<String>,
    pub designators: Vec<String>,
    pub quantity: u32,
    pub unit_price: Option<f64>,
    pub supplier: Option<String>,
    pub url: Option<String>,
    pub grams: Option<f64>,
    pub picked: bool,
    pub estimated: bool,
    pub total: Option<f64>,
}

/// Sums of the picked lines, by origin.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Totals {
    pub electronics: f64,
    pub printed: f64,
    pub manual: f64,
    pub total: f64,
    pub unpriced: usize,
    pub estimated: usize,
}

/// The priced bill of materials.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Report {
    pub currency: String,
    pub price_per_kg: f64,
    pub lines: Vec<Line>,
    pub totals: Totals,
}

fn merge(parts: Vec<Part>) -> BTreeMap<String, Part> {
    let mut merged: BTreeMap<String, Part> = BTreeMap::new();
    for part in parts {
        match merged.get_mut(&part.id) {
            Some(group) => group.designators.extend(part.designators),
            None => {
                merged.insert(part.id.clone(), part);
            }
        }
    }
    merged
}

fn cents(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn line(id: &str, kind: Kind, entry: &PartEntry) -> Line {
    Line {
        id: id.into(),
        kind,
        description: entry.description.clone().unwrap_or_else(|| id.into()),
        value: None,
        mpn: entry.mpn.clone(),
        designators: Vec::new(),
        quantity: entry.quantity.unwrap_or(1),
        unit_price: entry.unit_price,
        supplier: entry.supplier.clone(),
        url: entry.url.clone(),
        grams: None,
        picked: entry.picked.unwrap_or(true),
        estimated: false,
        total: None,
    }
}

fn detected_line(part: Part, entry: &PartEntry) -> Line {
    let quantity = u32::try_from(part.designators.len()).unwrap_or(u32::MAX);
    let fallback = [Some(part.kind.clone()), part.value.clone()];
    let named = fallback.into_iter().flatten().collect::<Vec<_>>().join(" ");
    let base = line(&part.id, Kind::Component, entry);
    let guess = match entry.unit_price {
        Some(_) => None,
        None => unit_price(&part.kind, part.value.as_deref(), part.mpn.as_deref()),
    };
    Line {
        unit_price: entry.unit_price.or(guess),
        estimated: guess.is_some(),
        description: entry.description.clone().unwrap_or(named),
        value: part.value,
        mpn: entry.mpn.clone().or(part.mpn),
        designators: part.designators,
        quantity: entry.quantity.unwrap_or(quantity),
        ..base
    }
}

fn print_line(body: &Body, entry: &PrintEntry, price_per_kg: f64) -> Line {
    let grams = entry.grams.or(body.grams);
    let base = PartEntry {
        description: Some(format!("3D print: {}", body.name)),
        quantity: entry.quantity,
        unit_price: grams.map(|g| g / 1000.0 * price_per_kg),
        picked: entry.picked,
        ..Default::default()
    };
    Line {
        grams,
        estimated: entry.grams.is_none() && body.grams.is_some(),
        ..line(&format!("print-{}", body.name), Kind::Print, &base)
    }
}

fn totals(lines: &mut [Line]) -> Totals {
    let mut sums = Totals::default();
    for line in lines.iter_mut() {
        line.total = line.unit_price.map(|p| cents(p * f64::from(line.quantity)));
        if !line.picked {
            continue;
        }
        sums.estimated += usize::from(line.estimated && line.total.is_some());
        let Some(total) = line.total else {
            sums.unpriced += 1;
            continue;
        };
        match line.kind {
            Kind::Component => sums.electronics += total,
            Kind::Print => sums.printed += total,
            Kind::Manual => sums.manual += total,
        }
    }
    sums.total = cents(sums.electronics + sums.printed + sums.manual);
    sums
}

/// Builds the report of `parts` and `bodies` with the prices and picks of `file`.
pub fn build(parts: Vec<Part>, bodies: &[Body], file: &BomFile) -> Report {
    let merged = merge(parts);
    let manual = file
        .parts
        .iter()
        .filter(|(id, _)| !merged.contains_key(*id));
    let manual: Vec<Line> = manual
        .map(|(id, entry)| line(id, Kind::Manual, entry))
        .collect();
    let default = PartEntry::default();
    let detected = merged.into_iter().map(|(id, part)| {
        let entry = file.parts.get(&id).unwrap_or(&default);
        detected_line(part, entry)
    });
    let printed = bodies.iter().map(|body| {
        let entry = file.prints.get(&body.name).cloned().unwrap_or_default();
        print_line(body, &entry, file.print.price_per_kg)
    });
    let mut lines: Vec<Line> = detected.chain(manual).chain(printed).collect();
    let totals = totals(&mut lines);
    Report {
        currency: file
            .currency
            .clone()
            .unwrap_or_else(|| DEFAULT_CURRENCY.into()),
        price_per_kg: file.print.price_per_kg,
        lines,
        totals,
    }
}
