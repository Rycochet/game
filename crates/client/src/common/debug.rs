//! Debug switches: dev tools on the server, debug info, detect monsters, god mode, truesight, kill mob, the mob Mob Mode makes.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::protocol::DevMob;

/// Whether the server accepts developer tools messages (from its `Welcome`).
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct ServerDevTools(pub bool);

/// Debug: open the debug menu's Teleport to World panel with this world's seed
/// in it (none: the world the player is in), as View Planet's button does.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeleportToWorldAsked {
    pub seed: Option<u64>,
}

/// Whether the status text's hour and view stats are on show (Show Debug
/// Info); its place, ping and frame rate always are.
#[derive(Resource, Clone, Copy, Debug)]
pub struct DebugInfo(pub bool);

impl Default for DebugInfo {
    fn default() -> Self {
        Self(true)
    }
}

/// Debug: every mob on the level is shown, seen or not.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DetectMonsters(pub bool);

/// Debug: mobs take no notice of the player, who can walk through them. As the
/// server is told; anything else god mode comes to do should go by this too.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GodMode(pub bool);

/// Debug: what is hidden is shown, such as what is secret about items. As the server
/// is told, as it is what sends them; anything else hidden should go by this too.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Truesight(pub bool);

/// Debug: no mob in the world thinks or moves (Freeze Mobs), as the server is told.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FreezeMobs(pub bool);

/// Debug: the mob a click puts on a cell in Mob Mode (`ClickMode::PlaceMob`),
/// as Create Mob's panel has it; the click gives its `x` and `y`.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreateMobOrder(pub DevMob);

impl Default for CreateMobOrder {
    fn default() -> Self {
        Self(DevMob {
            kind: CreatureKind::Goblin,
            x: 0,
            y: 0,
            attitude: None,
            gear: true,
            still: false,
            seed: 1,
        })
    }
}

/// Debug: the mob Inspect Mob last asked about and what the server said of it,
/// shown in its panel and asked again while that is open.
#[derive(Resource, Default, Clone, Debug, PartialEq, Eq)]
pub struct InspectedMob {
    pub mob: Option<Id>,
    pub lines: Vec<String>,
}
