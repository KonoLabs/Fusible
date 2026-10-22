//! Minimal S-expression tokenizer and parser for KiCad board files.

pub use super::footprint::{extract_footprints, KicadFootprint};

#[derive(Debug, Clone, PartialEq)]
pub enum Sexpr {
    Atom(String),
    List(Vec<Sexpr>),
}

impl Sexpr {
    pub fn as_atom(&self) -> Option<&str> {
        match self {
            Sexpr::Atom(s) => Some(s),
            Sexpr::List(_) => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Sexpr]> {
        match self {
            Sexpr::List(l) => Some(l),
            Sexpr::Atom(_) => None,
        }
    }
}

const MAX_DEPTH: usize = 512;

pub fn parse_sexpr(input: &str) -> Result<Sexpr, String> {
    let tokens = tokenize(input)?;
    let mut pos = 0;
    if tokens.is_empty() {
        return Err("empty S-expression input".to_string());
    }
    parse_expr(&tokens, &mut pos, 0)
}

/// Splits input into `(`, `)`, quoted strings (kept with their quotes) and bare atoms.
fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            c if c.is_whitespace() => i += 1,
            '(' | ')' => {
                tokens.push(chars[i].to_string());
                i += 1;
            }
            '"' => tokens.push(read_quoted(&chars, &mut i)?),
            _ => tokens.push(read_atom(&chars, &mut i)),
        }
    }

    Ok(tokens)
}

/// Reads a quoted string starting at the opening quote, resolving backslash escapes.
fn read_quoted(chars: &[char], i: &mut usize) -> Result<String, String> {
    *i += 1;
    let mut s = String::new();
    while *i < chars.len() && chars[*i] != '"' {
        if chars[*i] == '\\' && *i + 1 < chars.len() {
            *i += 1;
        }
        s.push(chars[*i]);
        *i += 1;
    }
    if *i >= chars.len() {
        return Err("unterminated string literal".to_string());
    }
    *i += 1;
    Ok(format!("\"{s}\""))
}

/// Reads a bare atom up to the next whitespace, parenthesis or quote.
fn read_atom(chars: &[char], i: &mut usize) -> String {
    let mut atom = String::new();
    while *i < chars.len()
        && !chars[*i].is_whitespace()
        && chars[*i] != '('
        && chars[*i] != ')'
        && chars[*i] != '"'
    {
        atom.push(chars[*i]);
        *i += 1;
    }
    atom
}

fn parse_expr(tokens: &[String], pos: &mut usize, depth: usize) -> Result<Sexpr, String> {
    if depth > MAX_DEPTH {
        return Err(format!(
            "S-expression nested deeper than {MAX_DEPTH} levels"
        ));
    }
    if *pos >= tokens.len() {
        return Err("unexpected end of tokens".to_string());
    }

    let tok = &tokens[*pos];
    if tok == "(" {
        return parse_list(tokens, pos, depth);
    }
    if tok == ")" {
        return Err("unexpected ')'".to_string());
    }
    *pos += 1;
    if tok.starts_with('"') && tok.ends_with('"') && tok.len() >= 2 {
        return Ok(Sexpr::Atom(tok[1..tok.len() - 1].to_string()));
    }
    Ok(Sexpr::Atom(tok.clone()))
}

/// Parses a list starting at its `(` token through the matching `)`.
fn parse_list(tokens: &[String], pos: &mut usize, depth: usize) -> Result<Sexpr, String> {
    *pos += 1;
    let mut list = Vec::new();
    while *pos < tokens.len() && tokens[*pos] != ")" {
        list.push(parse_expr(tokens, pos, depth + 1)?);
    }
    if *pos >= tokens.len() {
        return Err("unclosed parenthesis".to_string());
    }
    *pos += 1;
    Ok(Sexpr::List(list))
}
