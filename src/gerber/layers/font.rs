//! Built-in single-stroke vector font for silkscreen text.
//!
//! Each glyph is a set of polylines on a 4 x 6 grid with the origin at the bottom left.
//! A polyline is written as concatenated `xy` digit pairs and polylines are separated by `/`.

pub(super) const GLYPH_WIDTH: f64 = 4.0;
pub(super) const GLYPH_HEIGHT: f64 = 6.0;
pub(super) const ADVANCE: f64 = 5.0;

const GLYPHS: &[(char, &str)] = &[
    ('0', "103041453616050110"),
    ('1', "152620/1030"),
    ('2', "05163645440040"),
    ('3', "05163645443313/334241301001"),
    ('4', "30360242"),
    ('5', "4606033342413000"),
    ('6', "36160501103041423303"),
    ('7', "064610"),
    ('8', "13040516364544331302011030414233"),
    ('9', "43130405163645413010"),
    ('A', "002640/1232"),
    ('B', "00063645443303/3342413000"),
    ('C', "4536160501103041"),
    ('D', "00062645412000"),
    ('E', "46060040/0333"),
    ('F', "460600/0333"),
    ('G', "45361605011030414323"),
    ('H', "0006/4046/0343"),
    ('I', "1636/2620/1030"),
    ('J', "4641301001"),
    ('K', "0006/460340"),
    ('L', "060040"),
    ('M', "0006244640"),
    ('N', "00064046"),
    ('O', "103041453616050110"),
    ('P', "00063645443303"),
    ('Q', "103041453616050110/2240"),
    ('R', "00063645443303/2340"),
    ('S', "453616050413334241301001"),
    ('T', "0646/2026"),
    ('U', "060110304146"),
    ('V', "062046"),
    ('W', "0610243046"),
    ('X', "0046/0640"),
    ('Y', "062346/2320"),
    ('Z', "06460040"),
    ('.', "0001"),
    ('-', "1333"),
    ('_', "0040"),
    ('/', "0046"),
    ('+', "1333/2224"),
    (' ', ""),
];

/// Returns the polylines of `ch` in grid units, or `None` when the font has no such glyph.
/// Lowercase letters use their uppercase glyph.
pub(super) fn glyph(ch: char) -> Option<Vec<Vec<[f64; 2]>>> {
    let ch = ch.to_ascii_uppercase();
    let (_, spec) = GLYPHS.iter().find(|(c, _)| *c == ch)?;
    Some(
        spec.split('/')
            .filter(|stroke| !stroke.is_empty())
            .map(|stroke| {
                let digits: Vec<f64> = stroke.bytes().map(|b| f64::from(b - b'0')).collect();
                digits.chunks(2).map(|xy| [xy[0], xy[1]]).collect()
            })
            .collect(),
    )
}
