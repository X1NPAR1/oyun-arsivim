//! Valve KeyValues (VDF/ACF) metin biçimi için küçük ayrıştırıcı.
//!
//! Biçim: `"anahtar" "değer"` veya `"anahtar" { ... }`. Kaçışlar (`\\`, `\"`, `\n`, `\t`) ve `//`
//! yorumları desteklenir. Steam dosyaları zaman zaman yarım yazılmış olabildiği için bozuk girdi
//! panik değil, açıklayıcı bir hata üretir.
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vdf {
    Str(String),
    Obj(BTreeMap<String, Vdf>),
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VdfError {
    #[error("beklenmeyen dosya sonu")]
    UnexpectedEof,
    #[error("{0}. karakterde beklenmeyen simge")]
    UnexpectedToken(usize),
}

impl Vdf {
    /// Anahtarlar Steam'de büyük/küçük harf duyarsızdır (`AppState` / `appstate`).
    pub fn get(&self, key: &str) -> Option<&Vdf> {
        match self {
            Vdf::Obj(map) => map
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v),
            Vdf::Str(_) => None,
        }
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(Vdf::Str(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &Vdf)> {
        let map = match self {
            Vdf::Obj(map) => Some(map),
            Vdf::Str(_) => None,
        };
        map.into_iter().flat_map(|m| m.iter())
    }
}

struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    _src: &'a str,
}

impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        while let Some(&c) = self.chars.get(self.pos) {
            if c.is_whitespace() {
                self.pos += 1;
            } else if c == '/' && self.chars.get(self.pos + 1) == Some(&'/') {
                while let Some(&c) = self.chars.get(self.pos) {
                    if c == '\n' {
                        break;
                    }
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    fn string(&mut self) -> Result<String, VdfError> {
        // Tırnaklı veya (nadiren) tırnaksız simge.
        let quoted = self.chars.get(self.pos) == Some(&'"');
        if quoted {
            self.pos += 1;
        }
        let mut out = String::new();
        loop {
            let c = *self.chars.get(self.pos).ok_or(VdfError::UnexpectedEof)?;
            if quoted {
                match c {
                    '"' => {
                        self.pos += 1;
                        return Ok(out);
                    }
                    '\\' => {
                        let next = *self
                            .chars
                            .get(self.pos + 1)
                            .ok_or(VdfError::UnexpectedEof)?;
                        out.push(match next {
                            'n' => '\n',
                            't' => '\t',
                            other => other,
                        });
                        self.pos += 2;
                    }
                    _ => {
                        out.push(c);
                        self.pos += 1;
                    }
                }
            } else {
                if c.is_whitespace() || c == '{' || c == '}' || c == '"' {
                    return Ok(out);
                }
                out.push(c);
                self.pos += 1;
            }
        }
    }

    fn object_body(&mut self, top_level: bool) -> Result<BTreeMap<String, Vdf>, VdfError> {
        let mut map = BTreeMap::new();
        loop {
            self.skip_ws();
            match self.chars.get(self.pos) {
                None if top_level => return Ok(map),
                None => return Err(VdfError::UnexpectedEof),
                Some('}') if !top_level => {
                    self.pos += 1;
                    return Ok(map);
                }
                Some('}') | Some('{') => return Err(VdfError::UnexpectedToken(self.pos)),
                Some(_) => {
                    let key = self.string()?;
                    self.skip_ws();
                    let value = match self.chars.get(self.pos) {
                        Some('{') => {
                            self.pos += 1;
                            Vdf::Obj(self.object_body(false)?)
                        }
                        Some(_) => Vdf::Str(self.string()?),
                        None => return Err(VdfError::UnexpectedEof),
                    };
                    map.insert(key, value);
                }
            }
        }
    }
}

pub fn parse(src: &str) -> Result<Vdf, VdfError> {
    let src = src.trim_start_matches('\u{feff}');
    let mut parser = Parser {
        chars: src.chars().collect(),
        pos: 0,
        _src: src,
    };
    Ok(Vdf::Obj(parser.object_body(true)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_objects_and_case_insensitive_keys() {
        let v =
            parse("\"AppState\"\n{\n\t\"appid\"\t\t\"730\"\n\t\"name\" \"Counter-Strike 2\"\n}")
                .unwrap();
        let app = v.get("appstate").unwrap();
        assert_eq!(app.str("AppID"), Some("730"));
        assert_eq!(app.str("name"), Some("Counter-Strike 2"));
    }

    #[test]
    fn handles_escapes_comments_and_bom() {
        let v = parse(
            "\u{feff}// yorum\n\"a\" { \"path\" \"D:\\\\Oyunlar\\\\Şahin\" \"q\" \"x\\\"y\" }",
        )
        .unwrap();
        let a = v.get("a").unwrap();
        assert_eq!(a.str("path"), Some("D:\\Oyunlar\\Şahin"));
        assert_eq!(a.str("q"), Some("x\"y"));
    }

    #[test]
    fn broken_input_is_an_error_not_a_panic() {
        assert_eq!(parse("\"a\" { \"b\" \"c\""), Err(VdfError::UnexpectedEof));
        assert!(parse("\"a\" { \"b\" ").is_err());
        assert!(parse("}").is_err());
    }

    #[test]
    fn empty_input_is_empty_object() {
        assert_eq!(parse("").unwrap(), Vdf::Obj(BTreeMap::new()));
    }
}
