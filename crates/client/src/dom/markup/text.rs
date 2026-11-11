//! Words in markup: spaces tidied as HTML does, and `&…;` read.

use std::borrow::Cow;

/// Runs of spaces and line breaks made one space, and none at either end, so
/// templates can be indented freely. `&nbsp;` is a space that stays.
pub(super) fn tidy(raw: &str) -> Result<String, String> {
    let joined = raw.split_ascii_whitespace().collect::<Vec<_>>().join(" ");
    Ok(decode(&joined)?.into_owned())
}

/// `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&nbsp;` and `&#…;` read as what
/// they stand for. Borrowed when there are none.
pub(super) fn decode(raw: &str) -> Result<Cow<'_, str>, String> {
    if !raw.contains('&') {
        return Ok(Cow::Borrowed(raw));
    }
    let mut decoded = String::with_capacity(raw.len());
    let mut rest = raw;
    while let Some(start) = rest.find('&') {
        decoded.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find(';') else {
            return Err("`&` must be written `&amp;`".into());
        };
        decoded.push(entity(&after[..end])?);
        rest = &after[end + 1..];
    }
    decoded.push_str(rest);
    Ok(Cow::Owned(decoded))
}

fn entity(name: &str) -> Result<char, String> {
    let number = |digits: &str, radix| {
        u32::from_str_radix(digits, radix)
            .ok()
            .and_then(char::from_u32)
            .ok_or_else(|| format!("`&{name};` is not a character"))
    };
    match name {
        "amp" => Ok('&'),
        "lt" => Ok('<'),
        "gt" => Ok('>'),
        "quot" => Ok('"'),
        "apos" => Ok('\''),
        "nbsp" => Ok('\u{a0}'),
        _ => match name.strip_prefix('#') {
            Some(code) => match code.strip_prefix('x') {
                Some(hex) => number(hex, 16),
                None => number(code, 10),
            },
            None if name.contains(' ') || name.is_empty() => {
                Err("`&` must be written `&amp;`".into())
            }
            None => Err(format!("`&{name};` is not known")),
        },
    }
}
