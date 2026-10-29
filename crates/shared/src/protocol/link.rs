//! A connection between a client and the server: an in-process channel of
//! postcard-encoded messages.
//!
//! Single player uses it directly. Carrying bytes rather than the messages
//! themselves means nothing can be shared between the two sides by accident,
//! and a network transport can later pass the same bytes to and from a
//! socket.

use std::marker::PhantomData;
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

use serde::Serialize;
use serde::de::DeserializeOwned;

use super::{ClientMessage, ServerMessage};

/// The client's end: sends `ClientMessage`s, receives `ServerMessage`s.
pub type ClientLink = Link<ClientMessage, ServerMessage>;
/// The server's end of one client's connection.
pub type ServerLink = Link<ServerMessage, ClientMessage>;

/// One end of a connection, sending `Out` messages and receiving `In`.
pub struct Link<Out, In> {
    outgoing: Sender<Vec<u8>>,
    // A `Receiver` may only be used by one thread at a time; the `Mutex`
    // makes that a rule the compiler can check, so a `Link` can be shared
    // (for example as a Bevy resource).
    incoming: Mutex<Receiver<Vec<u8>>>,
    // Remembers the message types without storing any (`fn` so it doesn't
    // affect whether a `Link` can move between threads).
    messages: PhantomData<fn(Out) -> In>,
}

/// Why nothing more can be received.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkError {
    /// The other end has gone.
    Closed,
    /// A message couldn't be decoded: the other side is a different version
    /// or isn't playing by the rules.
    Garbled,
}

/// Both ends of a new in-process connection.
pub fn local_pair() -> (ClientLink, ServerLink) {
    let (to_server, from_client) = mpsc::channel();
    let (to_client, from_server) = mpsc::channel();
    (
        Link::new(to_server, from_server),
        Link::new(to_client, from_client),
    )
}

impl<Out: Serialize, In: DeserializeOwned> Link<Out, In> {
    fn new(outgoing: Sender<Vec<u8>>, incoming: Receiver<Vec<u8>>) -> Self {
        Self {
            outgoing,
            incoming: Mutex::new(incoming),
            messages: PhantomData,
        }
    }

    /// Sends a message. `false` if the other end has gone.
    pub fn send(&self, message: &Out) -> bool {
        let bytes = postcard::to_allocvec(message).expect("messages always encode");
        self.outgoing.send(bytes).is_ok()
    }

    /// The next message that has arrived, or `None` if there isn't one yet.
    pub fn receive(&self) -> Result<Option<In>, LinkError> {
        let incoming = self.incoming.lock().expect("never poisoned");
        match incoming.try_recv() {
            Ok(bytes) => postcard::from_bytes(&bytes)
                .map(Some)
                .map_err(|_| LinkError::Garbled),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(LinkError::Closed),
        }
    }
}
