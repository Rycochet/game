//! The character being made, as chosen so far.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::difficulty::Difficulty;
use shared::gender::Gender;
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
    /// Neutral until chosen.
    pub gender: Gender,
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
            gender: Gender::Neutral,
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
        Attributes::for_kind(self.kind, self.gender, &self.build)
    }

    /// What the kind and gender add on top, for the charts' bonus layer.
    pub fn bonus(&self) -> Attributes {
        let (kind, gender) = (self.kind.def().bonus(), self.gender.bonus());
        let mut bonus = Attributes::default();
        for which in shared::sheet::Attribute::ALL {
            bonus.set(which, kind.get(which) + gender.get(which));
        }
        bonus
    }

    /// Another kind: its looks start again, as each kind has its own.
    pub fn choose_kind(&mut self, kind: CreatureKind) {
        if self.kind != kind {
            self.kind = kind;
            self.appearance = Appearance::default();
        }
    }

    /// Another gender: a look it can't have (a female's beard) goes.
    pub fn choose_gender(&mut self, gender: Gender) {
        self.gender = gender;
        self.appearance.fit(gender);
    }

    /// Whether it can be sent: it needs a name that can be one.
    pub fn ready(&self) -> bool {
        shared::name::checked(&self.name).is_ok()
    }

    pub fn message(&self) -> ClientMessage {
        ClientMessage::CreateCharacter(NewCharacter {
            name: shared::name::checked(&self.name).unwrap_or_default(),
            kind: self.kind,
            gender: self.gender,
            appearance: self.appearance.clone(),
            build: self.build,
            difficulty: self.difficulty,
        })
    }
}

#[cfg(test)]
mod tests;
