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

fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\r' | '\n' => {
                i += 1;
            }
            '(' => {
                tokens.push("(".to_string());
                i += 1;
            }
            ')' => {
                tokens.push(")".to_string());
                i += 1;
            }
            '"' => {
                i += 1;
                let mut s = String::new();
                while i < chars.len() && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 1;
                        s.push(chars[i]);
                    } else {
                        s.push(chars[i]);
                    }
                    i += 1;
                }
                if i >= chars.len() {
                    return Err("unterminated string literal".to_string());
                }
                i += 1;
                tokens.push(format!("\"{s}\""));
            }
            _ => {
                let mut atom = String::new();
                while i < chars.len()
                    && !chars[i].is_whitespace()
                    && chars[i] != '('
                    && chars[i] != ')'
                    && chars[i] != '"'
                {
                    atom.push(chars[i]);
                    i += 1;
                }
                tokens.push(atom);
            }
        }
    }

    Ok(tokens)
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
    } else if tok == ")" {
        Err("unexpected ')'".to_string())
    } else if tok.starts_with('"') && tok.ends_with('"') && tok.len() >= 2 {
        *pos += 1;
        let inner = &tok[1..tok.len() - 1];
        Ok(Sexpr::Atom(inner.to_string()))
    } else {
        *pos += 1;
        Ok(Sexpr::Atom(tok.clone()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KicadFootprint {
    pub designator: String,
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
    pub layer: String,
}

pub fn extract_footprints(sexpr: &Sexpr) -> Vec<KicadFootprint> {
    let mut footprints = Vec::new();
    walk_sexpr_for_footprints(sexpr, &mut footprints);
    footprints
}

fn walk_sexpr_for_footprints(node: &Sexpr, out: &mut Vec<KicadFootprint>) {
    if let Sexpr::List(list) = node {
        if let Some(first) = list.first().and_then(|f| f.as_atom()) {
            if first == "footprint" || first == "module" {
                if let Some(fp) = parse_footprint_node(list) {
                    out.push(fp);
                }
            }
        }

        for item in list {
            walk_sexpr_for_footprints(item, out);
        }
    }
}

fn parse_footprint_node(list: &[Sexpr]) -> Option<KicadFootprint> {
    let mut designator = None;
    let mut x = 0.0;
    let mut y = 0.0;
    let mut rotation = 0.0;
    let mut layer = "top".to_string();

    for item in list {
        if let Sexpr::List(sub) = item {
            if let Some(tag) = sub.first().and_then(|t| t.as_atom()) {
                match tag {
                    "at" => {
                        if sub.len() >= 3 {
                            x = sub[1].as_atom().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                            y = sub[2].as_atom().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                        }
                        if sub.len() >= 4 {
                            rotation = sub[3].as_atom().and_then(|v| v.parse().ok()).unwrap_or(0.0);
                        }
                    }
                    "layer" => {
                        if sub.len() >= 2 {
                            if let Some(l) = sub[1].as_atom() {
                                if l.contains("B.Cu") {
                                    layer = "bottom".to_string();
                                } else {
                                    layer = "top".to_string();
                                }
                            }
                        }
                    }
                    "property" if sub.len() >= 3 && sub[1].as_atom() == Some("Reference") => {
                        designator = sub[2].as_atom().map(|s| s.to_string());
                    }
                    "fp_text" if sub.len() >= 3 && sub[1].as_atom() == Some("reference") => {
                        designator = sub[2].as_atom().map(|s| s.to_string());
                    }
                    _ => {}
                }
            }
        }
    }

    let des = designator?;
    Some(KicadFootprint {
        designator: des,
        x,
        y,
        rotation,
        layer,
    })
}
