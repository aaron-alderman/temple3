//! Bounded source-receipt JSON, separate from number-free constitutional JSON.
//! Strings retain UTF-16 code units so lone surrogates are never replaced before hashing.
use crate::{Result, err, require};

#[derive(Clone, Debug)]
pub enum SourceValue {
    Null,
    Bool(bool),
    Number(f64),
    String(Vec<u16>),
    Array(Vec<SourceValue>),
    Object(Vec<(Vec<u16>, SourceValue)>),
}
impl PartialEq for SourceValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Array(a), Self::Array(b)) => a == b,
            (Self::Object(a), Self::Object(b)) => {
                a.len() == b.len()
                    && a.iter()
                        .all(|(k, v)| b.iter().any(|(key, value)| key == k && value == v))
            }
            _ => false,
        }
    }
}
impl SourceValue {
    pub fn field(&self, name: &str) -> Result<&Self> {
        let key: Vec<_> = name.encode_utf16().collect();
        if let Self::Object(fields) = self {
            return fields
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v)
                .ok_or_else(|| err("source-json-field"));
        }
        Err(err("source-json-object"))
    }
    pub fn array(&self) -> Result<&[Self]> {
        if let Self::Array(values) = self {
            Ok(values)
        } else {
            Err(err("source-json-array"))
        }
    }
    pub fn text(&self) -> Result<String> {
        if let Self::String(value) = self {
            String::from_utf16(value).map_err(|_| err("source-json-unicode"))
        } else {
            Err(err("source-json-string"))
        }
    }
}

fn number(value: f64) -> Result<String> {
    require(value.is_finite(), "source-seal-value")?;
    if value == 0.0 {
        return Ok("0".into());
    }
    let raw = value.abs().to_string();
    let (mantissa, exponent) = raw.split_once('e').unwrap_or((&raw, "0"));
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let mut digits = format!("{whole}{fraction}");
    let mut point = whole.len() as i32
        + exponent
            .parse::<i32>()
            .map_err(|_| err("source-json-number"))?;
    while digits.starts_with('0') {
        digits.remove(0);
        point -= 1;
    }
    while digits.ends_with('0') {
        digits.pop();
    }
    let result = if point > 0 && point <= 21 {
        if point as usize >= digits.len() {
            format!("{}{}", digits, "0".repeat(point as usize - digits.len()))
        } else {
            format!(
                "{}.{}",
                &digits[..point as usize],
                &digits[point as usize..]
            )
        }
    } else if point > -6 && point <= 0 {
        format!("0.{}{}", "0".repeat((-point) as usize), digits)
    } else {
        let exp = point - 1;
        format!(
            "{}{}e{}{exp}",
            &digits[..1],
            if digits.len() > 1 {
                format!(".{}", &digits[1..])
            } else {
                String::new()
            },
            if exp >= 0 { "+" } else { "" }
        )
    };
    Ok(format!("{}{result}", if value < 0.0 { "-" } else { "" }))
}
fn quoted(units: &[u16]) -> String {
    let mut result = String::from("\"");
    for decoded in char::decode_utf16(units.iter().copied()) {
        match decoded {
            Err(e) => result.push_str(&format!("\\u{:04x}", e.unpaired_surrogate())),
            Ok(c) => match c {
                '"' => result.push_str("\\\""),
                '\\' => result.push_str("\\\\"),
                '\u{8}' => result.push_str("\\b"),
                '\u{c}' => result.push_str("\\f"),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                c if c < '\u{20}' => result.push_str(&format!("\\u{:04x}", c as u32)),
                c => result.push(c),
            },
        }
    }
    result.push('"');
    result
}
pub fn canonical(value: &SourceValue) -> Result<String> {
    fn encode(v: &SourceValue, depth: usize) -> Result<String> {
        require(depth <= 32, "source-seal-depth")?;
        Ok(match v {
            SourceValue::Null => "null".into(),
            SourceValue::Bool(b) => b.to_string(),
            SourceValue::Number(n) => number(*n)?,
            SourceValue::String(s) => quoted(s),
            SourceValue::Array(values) => format!(
                "[{}]",
                values
                    .iter()
                    .map(|v| encode(v, depth + 1))
                    .collect::<Result<Vec<_>>>()?
                    .join(",")
            ),
            SourceValue::Object(fields) => {
                let mut fields: Vec<_> = fields.iter().collect();
                fields.sort_by_cached_key(|(key, _)| String::from_utf16_lossy(key).into_bytes());
                format!(
                    "{{{}}}",
                    fields
                        .into_iter()
                        .map(|(key, v)| Ok(format!("{}:{}", quoted(key), encode(v, depth + 1)?)))
                        .collect::<Result<Vec<_>>>()?
                        .join(",")
                )
            }
        })
    }
    encode(value, 0)
}

struct Parser<'a> {
    text: &'a str,
    at: usize,
}
impl Parser<'_> {
    fn whitespace(&mut self) {
        while self
            .text
            .as_bytes()
            .get(self.at)
            .is_some_and(|b| b" \r\n\t".contains(b))
        {
            self.at += 1;
        }
    }
    fn byte(&mut self, b: u8) -> Result<()> {
        require(
            self.text.as_bytes().get(self.at) == Some(&b),
            "source-json-syntax",
        )?;
        self.at += 1;
        Ok(())
    }
    fn string(&mut self) -> Result<Vec<u16>> {
        self.byte(b'"')?;
        let mut result = Vec::new();
        loop {
            let c = self.text[self.at..]
                .chars()
                .next()
                .ok_or_else(|| err("source-json-string"))?;
            self.at += c.len_utf8();
            match c {
                '"' => return Ok(result),
                '\\' => {
                    let escaped = *self
                        .text
                        .as_bytes()
                        .get(self.at)
                        .ok_or_else(|| err("source-json-string"))?;
                    self.at += 1;
                    match escaped {
                        b'"' | b'\\' | b'/' => result.push(escaped as u16),
                        b'b' => result.push(8),
                        b'f' => result.push(12),
                        b'n' => result.push(10),
                        b'r' => result.push(13),
                        b't' => result.push(9),
                        b'u' => {
                            let part = self
                                .text
                                .as_bytes()
                                .get(self.at..self.at + 4)
                                .ok_or_else(|| err("source-json-string"))?;
                            require(part.iter().all(u8::is_ascii_hexdigit), "source-json-string")?;
                            let part =
                                std::str::from_utf8(part).map_err(|_| err("source-json-string"))?;
                            result.push(
                                u16::from_str_radix(part, 16)
                                    .map_err(|_| err("source-json-string"))?,
                            );
                            self.at += 4;
                        }
                        _ => return Err(err("source-json-string")),
                    }
                }
                c => {
                    require(c >= '\u{20}', "source-json-string")?;
                    let mut units = [0; 2];
                    result.extend_from_slice(c.encode_utf16(&mut units));
                }
            }
        }
    }
    fn value(&mut self, depth: usize) -> Result<SourceValue> {
        require(depth <= 32, "source-seal-depth")?;
        self.whitespace();
        let b = *self
            .text
            .as_bytes()
            .get(self.at)
            .ok_or_else(|| err("source-json-syntax"))?;
        match b {
            b'"' => Ok(SourceValue::String(self.string()?)),
            b'[' => {
                self.at += 1;
                self.whitespace();
                let mut values = Vec::new();
                if self.text.as_bytes().get(self.at) != Some(&b']') {
                    loop {
                        values.push(self.value(depth + 1)?);
                        self.whitespace();
                        if self.text.as_bytes().get(self.at) == Some(&b']') {
                            break;
                        }
                        self.byte(b',')?;
                    }
                }
                self.byte(b']')?;
                Ok(SourceValue::Array(values))
            }
            b'{' => {
                self.at += 1;
                self.whitespace();
                let mut fields = Vec::new();
                if self.text.as_bytes().get(self.at) != Some(&b'}') {
                    loop {
                        self.whitespace();
                        let key = self.string()?;
                        require(
                            !fields.iter().any(|(k, _)| *k == key),
                            "source-json-duplicate-key",
                        )?;
                        self.whitespace();
                        self.byte(b':')?;
                        fields.push((key, self.value(depth + 1)?));
                        self.whitespace();
                        if self.text.as_bytes().get(self.at) == Some(&b'}') {
                            break;
                        }
                        self.byte(b',')?;
                    }
                }
                self.byte(b'}')?;
                Ok(SourceValue::Object(fields))
            }
            b't' | b'f' | b'n' => {
                let (word, v) = match b {
                    b't' => ("true", SourceValue::Bool(true)),
                    b'f' => ("false", SourceValue::Bool(false)),
                    _ => ("null", SourceValue::Null),
                };
                require(self.text[self.at..].starts_with(word), "source-json-syntax")?;
                self.at += word.len();
                Ok(v)
            }
            _ => {
                let start = self.at;
                while self
                    .text
                    .as_bytes()
                    .get(self.at)
                    .is_some_and(|b| b"0123456789.eE+-".contains(b))
                {
                    self.at += 1;
                }
                let raw = &self.text[start..self.at];
                // serde's JSON number grammar rejects leading zeros and incomplete exponents;
                // std's correctly rounded binary64 parser preserves the source Number value.
                require(
                    serde_json::from_str::<serde_json::Number>(raw).is_ok(),
                    "source-json-number",
                )?;
                let value = raw.parse::<f64>().map_err(|_| err("source-json-number"))?;
                require(value.is_finite(), "source-seal-value")?;
                Ok(SourceValue::Number(value))
            }
        }
    }
}
pub fn parse(text: &str) -> Result<SourceValue> {
    require(text.len() <= 8 * 1024 * 1024, "source-json-size")?;
    let mut parser = Parser { text, at: 0 };
    let value = parser.value(0)?;
    parser.whitespace();
    require(parser.at == text.len(), "source-json-trailing")?;
    Ok(value)
}
