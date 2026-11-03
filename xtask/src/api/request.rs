//! What to send where, and JSON values from the command line.

use super::CLIENT_PORT;

/// What to send where.
#[derive(Debug, PartialEq)]
pub(super) struct Request {
    pub(super) port: u16,
    pub(super) method: String,
    /// JSON, or none.
    pub(super) params: Option<String>,
}

impl Request {
    pub(super) fn parse(args: &[String]) -> Result<Self, String> {
        let mut port = CLIENT_PORT;
        let mut rest = args.iter();
        let method = loop {
            match rest.next().map(String::as_str) {
                Some("--server") => port = CLIENT_PORT + 1,
                Some("--port") => {
                    port = rest
                        .next()
                        .and_then(|port| port.parse().ok())
                        .ok_or("--port needs a port number")?;
                }
                Some(method) => break method.to_owned(),
                None => return Err("Which method?".into()),
            }
        };
        let rest: Vec<&String> = rest.collect();
        let params = match rest.as_slice() {
            [] => None,
            [json] if json.starts_with('{') || json.starts_with('[') => Some(json.to_string()),
            pairs => {
                let fields = pairs
                    .iter()
                    .map(|pair| {
                        let (key, value) = pair
                            .split_once('=')
                            .ok_or_else(|| format!("{pair} isn't key=value"))?;
                        Ok(format!("{}:{}", quote(key), json_value(value)))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                Some(format!("{{{}}}", fields.join(",")))
            }
        };
        Ok(Self {
            port,
            method,
            params,
        })
    }

    /// The JSON-RPC request.
    pub(super) fn body(&self) -> String {
        let params = self
            .params
            .as_ref()
            .map(|params| format!(",\"params\":{params}"))
            .unwrap_or_default();
        format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":{}{params}}}",
            quote(&self.method)
        )
    }
}

/// A value from the command line: as it is if it looks like JSON (numbers,
/// `true`, `false`, `null`, objects, lists, strings), otherwise as a string.
pub(super) fn json_value(value: &str) -> String {
    let looks_like_json = value.parse::<f64>().is_ok()
        || matches!(value, "true" | "false" | "null")
        || value.starts_with(['{', '[', '"']);
    if looks_like_json {
        value.to_owned()
    } else {
        quote(value)
    }
}

/// A JSON string.
pub(super) fn quote(text: &str) -> String {
    let mut quoted = String::from('"');
    for c in text.chars() {
        match c {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            c if (c as u32) < 0x20 => quoted.push_str(&format!("\\u{:04x}", c as u32)),
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}
