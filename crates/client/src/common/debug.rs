//! Debug switches: dev tools on the server, debug info, detect monsters, god mode, truesight, kill mob.

use bevy::prelude::*;

/// Whether the server accepts developer tools messages (from its `Welcome`).
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct ServerDevTools(pub bool);

/// Whether the status text with position, hour, ping and stats is on show.
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

/// Debug: clicking a mob kills it. Only this client knows it is on; the server checks
/// each kill.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KillMob(pub bool);
