//! The character being made, as chosen so far.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::difficulty::Difficulty;
use shared::protocol::{ClientMessage, NewCharacter};
use shared::sheet::{Appearance, Attributes, Build, Row};

/// What has been chosen so far. Only the server's word counts: this is what is
/// sent, and what the screen shows meanwhile.
#[derive(Resource, Clone, Debug, PartialEq)]
pub(super) struct Draft {
    pub name: String,
    /// The last name suggested, to tell whether the player has typed their own.
    pub suggested: String,
    pub kind: CreatureKind,
    pub appearance: Appearance,
    pub build: Build,
    /// The triangle the keys move the dot of.
    pub chosen: Row,
    /// How hard the game is to be for them: the only time Ironman is chosen.
    pub difficulty: Difficulty,
}

impl Default for Draft {
    fn default() -> Self {
        Self {
            name: String::new(),
            suggested: String::new(),
            kind: CreatureKind::Human,
            appearance: Appearance::default(),
            build: Build::CENTRED,
            chosen: Row::Body,
            difficulty: Difficulty::Normal,
        }
    }
}

impl Draft {
    /// The kinds that can be played, in the order the kind list shows them.
    pub fn playable() -> Vec<CreatureKind> {
        CreatureKind::ALL
            .into_iter()
            .filter(|kind| kind.def().playable)
            .collect()
    }

    /// What these choices give, worked out as the server will.
    pub fn attributes(&self) -> Attributes {
        Attributes::for_kind(self.kind, &self.build)
    }

    /// Another kind: its looks start again, as each kind has its own.
    pub fn choose_kind(&mut self, kind: CreatureKind) {
        if self.kind != kind {
            self.kind = kind;
            self.appearance = Appearance::default();
        }
    }

    /// Whether it can be sent: it needs a name.
    pub fn ready(&self) -> bool {
        !self.name.trim().is_empty()
    }

    pub fn message(&self) -> ClientMessage {
        ClientMessage::CreateCharacter(NewCharacter {
            name: self.name.trim().to_owned(),
            kind: self.kind,
            appearance: self.appearance.clone(),
            build: self.build,
            difficulty: self.difficulty,
        })
    }
}

#[cfg(test)]
mod tests;
