//! Why a client speaking another protocol is refused, saying which side is out
//! of date. Shown after "The server refused: ".

use shared::protocol::PROTOCOL_VERSION;

/// The refusal for a client that speaks protocol `theirs` (not this server's).
pub(super) fn other_protocol(theirs: u32) -> String {
    if theirs > PROTOCOL_VERSION {
        // Reloading the page wouldn't help: it is the server that is behind.
        format!(
            "this server is older than your game (protocol {PROTOCOL_VERSION}, not \
             {theirs}): the server needs updating"
        )
    } else {
        format!(
            "your game is older than this server (protocol {theirs}, not \
             {PROTOCOL_VERSION}): update it, or reload the page in a browser"
        )
    }
}

#[cfg(test)]
mod tests;
