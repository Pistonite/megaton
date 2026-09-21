
use cu::pre::*;

/// Dump a value given the key
pub fn dump(key: &str, mut value: json::Value) -> cu::Result<json::Value> {
        let mut rest = key;
        let mut current_path = String::new();
        loop {
            let next = cu::check!(
                parse_next_key(rest),
                "error parsing config key"
            )?;
            let key = match next {
                Some((key, next_rest)) => {
                    rest = next_rest;
                    key
                }
                None => return Ok(value),
            };
            match value {
                json::Value::Object(mut map) => match map.remove(key) {
                    None => {
                        let keys = format!("{:?}", map.keys().collect::<Vec<_>>());
                        cu::bail!(
                            "invalid key '{key}' at '{current_path}', valid keys are: {keys}"
                        );
                    }
                    Some(v) => value = v,
                },
                json::Value::Array(mut values) => {
                    let len = values.len();
                    let num = cu::parse::<usize>(key.trim());
                    let num = cu::check!(
                        num,
                        "invalid key '{key}' at '{current_path}', valid keys are: [0 - {}] for array of length {len}",
                        len - 1
                    )?;
                    if num >= len {
                        cu::bail!("index {num} out of bound: length at '{current_path}' is {len}");
                    }
                    value = values.swap_remove(num);
                }
                x => {
                    let stringified = match json::stringify(&x) {
                        Ok(x) => x,
                        Err(_) => format!("{x:?}"),
                    };
                    cu::bail!(
                        "invalid key '{key}' at '{current_path}', is a scalar value: {stringified}"
                    );
                }
            }
            if !current_path.is_empty() {
                current_path.push('.');
            }
            current_path.push_str(key);
        }
}

/// Parse the next key segment
fn parse_next_key(key: &str) -> cu::Result<Option<(&str, &str)>> {
    let key = trim_rest(key);
    if key.is_empty() {
        return Ok(None);
    }
    let c1 = cu::some!(key.chars().next()) ;
    match c1 {
        '"' => {
            let i = key[1..].find('"');
            let i =cu::check!(i, "unclosed double-quote in key: {key}")?;
            Ok(Some((&key[1..i+1], trim_rest(&key[i+2..]))))
        }
        '\'' => {
            let i = key[1..].find('\'');
            let i =cu::check!(i, "unclosed single-quote in key: {key}")?;
            Ok(Some((&key[1..i+1], trim_rest(&key[i+2..]))))
        }
        _ => {
            match key.find('.') {
                None => Ok(Some((key.trim(), ""))),
                Some(i) => {
                    Ok(Some((key[..i].trim(),trim_rest(&key[i+1..]))))
                }
            }
        }
    }
}

fn trim_rest(key: &str) -> &str {
    let mut key = key.trim_start();
    while key.starts_with('.') {
        key = key[1..].trim_start();
    }
    key
}

#[derive(clap::ValueEnum, Default, Clone, Copy)]
pub enum DumpFormat {
    /// One-line JSON
    Json,
    /// Pretty JSON
    JsonPretty,
    /// Raw: arrays are dumped as one value per line; objects are dumped as one `key=value` per
    /// line; over-complex objects cannot be dumped
    #[default]
    Raw,
    /// Like Raw, but array and objects are space-separated instead of one per line.
    OneLine
}
impl DumpFormat {
    pub fn stringify(self, value: &json::Value, hex: bool) -> cu::Result<String> {
        match self {
            Self::Json => {
                json::stringify(&value)
            }
            Self::JsonPretty => {
                json::stringify_pretty(&value)
            }
            Self::Raw => {
                json_obj_to_raw(&value, '\n', hex)
            }
            Self::OneLine => {
                json_obj_to_raw(&value, ' ', hex)
            }
        }
    }
}

fn json_obj_to_raw(value: &json::Value, join: char, hex: bool) -> cu::Result<String> {
    let mut buf = String::new();
    match value {
        json::Value::Array(values) => {
            for v in values {
                if !buf.is_empty() {
                    buf.push(join);
                }
                json_to_raw(&mut buf, v, hex)?;
            }
        }
        json::Value::Object(map) => {
            for (k,v) in map {
                if !buf.is_empty() {
                    buf.push(join);
                }
                buf.push_str(k);
                buf.push('=');
                json_to_raw(&mut buf, v, hex)?;
            }
        }
        other => {
            json_to_raw(&mut buf, other, hex)?;
        }
    }
    Ok(buf)
}

fn json_to_raw(out: &mut String, value: &json::Value, hex: bool) -> cu::Result<()> {
    match value {
        json::Value::Object(_) | json::Value::Array(_) => {
            cu::bail!("object is too complex; please use --format=json or --format=json-pretty");
        }
        json::Value::Null => out.push_str("null"),
        json::Value::Bool(x) => {
            if *x {
                out .push_str( "true");
            } else {
                out .push_str( "false");
            }
        }
        json::Value::Number(n) => {
            use std::fmt::Write;
            if hex && let Some(x) = n.as_u64() {
                let _ = write!(out, "0x{x:x}");
            } else {
                let _ = write!(out, "{n}");
            }
        }
        json::Value::String(x) => {
            out.push_str(x)
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    /// Parse a whole key path into its segments
    fn parse_all(mut key: &str) -> cu::Result<Vec<&str>> {
        let mut segments = Vec::new();
        while let Some((segment, rest)) = parse_next_key(key)? {
            segments.push(segment);
            key = rest;
        }
        Ok(segments)
    }

    #[test]
    fn empty_key() -> cu::Result<()> {
        assert_eq!(parse_next_key("")?, None);
        assert_eq!(parse_next_key("   ")?, None);
        // separators alone are not a segment
        assert_eq!(parse_next_key(".")?, None);
        assert_eq!(parse_next_key(" . . ")?, None);
        Ok(())
    }

    #[test]
    fn single_segment() -> cu::Result<()> {
        assert_eq!(parse_next_key("foo")?, Some(("foo", "")));
        // surrounding whitespace is not part of the segment
        assert_eq!(parse_next_key("  foo  ")?, Some(("foo", "")));
        Ok(())
    }

    #[test]
    fn multiple_segments() -> cu::Result<()> {
        assert_eq!(parse_next_key("foo.bar.baz")?, Some(("foo", "bar.baz")));
        assert_eq!(parse_next_key("bar.baz")?, Some(("bar", "baz")));
        assert_eq!(parse_next_key("baz")?, Some(("baz", "")));
        Ok(())
    }

    #[test]
    fn separators_are_trimmed() -> cu::Result<()> {
        // leading and repeated separators are skipped
        assert_eq!(parse_next_key(".foo.bar")?, Some(("foo", "bar")));
        assert_eq!(parse_next_key("foo..bar")?, Some(("foo", "bar")));
        assert_eq!(parse_next_key(" . foo . bar ")?, Some(("foo", "bar ")));
        // a trailing separator leaves nothing to parse
        assert_eq!(parse_next_key("foo.")?, Some(("foo", "")));
        Ok(())
    }

    #[test]
    fn double_quoted_segment() -> cu::Result<()> {
        // a quoted segment may contain separators and whitespace
        assert_eq!(parse_next_key(r#""foo.bar".baz"#)?, Some(("foo.bar", "baz")));
        assert_eq!(parse_next_key(r#"" foo ".bar"#)?, Some((" foo ", "bar")));
        assert_eq!(parse_next_key(r#""foo""#)?, Some(("foo", "")));
        assert_eq!(parse_next_key(r#"  "foo" . bar"#)?, Some(("foo", "bar")));
        // an empty quoted segment is an empty key
        assert_eq!(parse_next_key(r#""".bar"#)?, Some(("", "bar")));
        // the other quote character is literal inside a quoted segment
        assert_eq!(parse_next_key(r#""fo'o".bar"#)?, Some(("fo'o", "bar")));
        Ok(())
    }

    #[test]
    fn single_quoted_segment() -> cu::Result<()> {
        assert_eq!(parse_next_key("'foo.bar'.baz")?, Some(("foo.bar", "baz")));
        assert_eq!(parse_next_key("' foo '.bar")?, Some((" foo ", "bar")));
        assert_eq!(parse_next_key("'foo'")?, Some(("foo", "")));
        assert_eq!(parse_next_key("  'foo' . bar")?, Some(("foo", "bar")));
        assert_eq!(parse_next_key("''.bar")?, Some(("", "bar")));
        assert_eq!(parse_next_key(r#"'fo"o'.bar"#)?, Some((r#"fo"o"#, "bar")));
        Ok(())
    }

    #[test]
    fn unclosed_quote_is_an_error() {
        assert!(parse_next_key(r#""foo"#).is_err());
        assert!(parse_next_key(r#""foo.bar"#).is_err());
        assert!(parse_next_key("'foo").is_err());
        // a quote only opens a segment at the start of one
        assert!(parse_next_key(r#"foo."bar"#).is_ok());
    }

    #[test]
    fn non_ascii_segments() -> cu::Result<()> {
        assert_eq!(parse_next_key("中文.键")?, Some(("中文", "键")));
        assert_eq!(parse_next_key(r#""中文.键".x"#)?, Some(("中文.键", "x")));
        Ok(())
    }

    #[test]
    fn whole_key_path() -> cu::Result<()> {
        assert_eq!(parse_all("")?, Vec::<&str>::new());
        assert_eq!(parse_all("foo.bar.baz")?, vec!["foo", "bar", "baz"]);
        assert_eq!(parse_all(" foo . bar ")?, vec!["foo", "bar"]);
        assert_eq!(
            parse_all(r#"profile."my.profile".'flags'.0"#)?,
            vec!["profile", "my.profile", "flags", "0"]
        );
        Ok(())
    }
}
