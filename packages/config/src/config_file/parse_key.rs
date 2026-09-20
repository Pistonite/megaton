use cu::pre::*;

/// Parse the next key segment
pub fn parse_next_key(key: &str) -> cu::Result<Option<(&str, &str)>> {
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
