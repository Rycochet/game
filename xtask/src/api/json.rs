//! Fields from an answer, and answers indented for reading.

/// A string field anywhere in `json`, by name (enough for `wait_for_file`).
pub(super) fn string_field(json: &str, name: &str) -> Option<String> {
    let start = json.find(&format!("\"{name}\":\""))? + name.len() + 4;
    let mut value = String::new();
    let mut chars = json[start..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(value),
            '\\' => match chars.next()? {
                'n' => value.push('\n'),
                't' => value.push('\t'),
                other => value.push(other),
            },
            c => value.push(c),
        }
    }
    None
}

/// A whole-number field anywhere in `json`, by name.
pub(super) fn number_field(json: &str, name: &str) -> Option<u64> {
    let start = json.find(&format!("\"{name}\":"))? + name.len() + 3;
    let digits: String = json[start..]
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

/// `json` indented, for reading.
pub(super) fn pretty(json: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let newline = |out: &mut String, depth: usize| {
        out.push('\n');
        out.push_str(&"  ".repeat(depth));
    };
    let mut chars = json.trim().chars().peekable();
    while let Some(c) = chars.next() {
        if in_string {
            out.push(c);
            match (escaped, c) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => in_string = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
            }
            // Empty ones stay on one line.
            '{' | '[' if chars.peek().is_some_and(|next| matches!(next, '}' | ']')) => {
                out.push(c);
                out.extend(chars.next());
            }
            '{' | '[' => {
                depth += 1;
                out.push(c);
                newline(&mut out, depth);
            }
            '}' | ']' => {
                depth = depth.saturating_sub(1);
                newline(&mut out, depth);
                out.push(c);
            }
            ',' => {
                out.push(c);
                newline(&mut out, depth);
            }
            ':' => out.push_str(": "),
            c if c.is_whitespace() => {}
            c => out.push(c),
        }
    }
    out
}
