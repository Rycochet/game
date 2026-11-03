//! Reading the key a player pastes in for a server: its certificate (PEM, as a
//! server's `/certificate` page or Server Options shows it) or its key's
//! fingerprint (as its log shows it).

/// The fingerprint of the key in `text`, or `None` if it holds neither a
/// certificate nor a fingerprint. Spaces, line breaks and colons don't matter.
pub fn server_key(text: &str) -> Option<String> {
    let squeezed: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ':')
        .collect();
    if squeezed.len() == 64 && squeezed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(squeezed.to_lowercase());
    }
    let body = squeezed
        .strip_prefix("-----BEGINCERTIFICATE-----")?
        .split("-----ENDCERTIFICATE-----")
        .next()?;
    let der = from_base64(body)?;
    Some(server::certificate_facts(&der)?.key_fingerprint)
}

/// Base64 (standard, with or without `=` padding) to bytes.
fn from_base64(text: &str) -> Option<Vec<u8>> {
    let value = |c: u8| match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    };
    let text = text.trim_end_matches('=');
    let mut bytes = Vec::with_capacity(text.len() * 3 / 4);
    let (mut bits, mut count) = (0u32, 0);
    for c in text.bytes() {
        bits = (bits << 6) | u32::from(value(c)?);
        count += 6;
        if count >= 8 {
            count -= 8;
            bytes.push((bits >> count) as u8);
        }
    }
    Some(bytes)
}

#[cfg(test)]
mod tests;
