//! Adding clients to the server's `Connections`, and asking about them.

use shared::protocol::{CharacterId, ServerLink, ServerMessage};

use super::{Client, Connections};
use crate::ServerWorlds;
use crate::accounts::Accounts;

impl Connections {
    pub(crate) fn new(accounts: Accounts) -> Self {
        Self {
            clients: Vec::new(),
            accounts,
        }
    }

    /// A newly connected client, which must log in.
    pub fn add(&mut self, link: ServerLink) {
        self.clients.push(Client::new(link, None));
    }

    /// A newly connected client that needs no login, playing this character:
    /// the embedded server's one player.
    pub fn add_playing(&mut self, link: ServerLink, character: CharacterId) {
        self.clients.push(Client::new(link, Some(character)));
    }

    /// The embedded server's own player, already logged in to their account,
    /// to choose a character. Refused if saves can't be reached, as anyone else
    /// would be.
    pub fn add_single_player(&mut self, link: ServerLink) {
        match self.accounts.single_player() {
            Ok(account) => self.clients.push(Client::host(link, account)),
            Err(error) => {
                let mut client = Client::new(link, None);
                client.refuse_for_saves(&error);
                client.gone = true;
                self.clients.push(client);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    /// Everyone connected, to change in tests and from the debug API.
    #[cfg(any(test, feature = "debug-api"))]
    pub(crate) fn clients_mut(&mut self) -> &mut [Client] {
        &mut self.clients
    }

    /// Everyone connected, for the debug API.
    #[cfg(feature = "debug-api")]
    pub(crate) fn clients(&self) -> &[Client] {
        &self.clients
    }

    /// What the host has asked of opening the server to others, if anything.
    pub(crate) fn take_online_request(
        &mut self,
    ) -> Option<Option<shared::protocol::OnlineSettings>> {
        let host = self.clients.iter_mut().find(|client| client.host)?;
        host.asked_online.take()
    }

    /// Tells the host (the embedded server's own player), if connected.
    pub(crate) fn tell_host(&self, message: &ServerMessage) {
        for client in self
            .clients
            .iter()
            .filter(|client| client.host && !client.gone)
        {
            client.link.send(message);
        }
    }

    /// Tells everyone but the host this, and disconnects them, saving them
    /// (the server is closing to others).
    pub(crate) fn send_others_away(&mut self, worlds: &mut ServerWorlds, message: &ServerMessage) {
        for client in self.clients.iter_mut().filter(|client| !client.host) {
            client.link.send(message);
            client.disconnect(worlds.of(client.world));
        }
        self.clients.retain(|client| !client.gone);
    }

    /// Disconnects everyone, saving them (the server is stopping).
    pub(crate) fn disconnect_all(&mut self, worlds: &mut ServerWorlds) {
        for client in &mut self.clients {
            client.disconnect(worlds.of(client.world));
        }
        self.clients.clear();
    }
}
