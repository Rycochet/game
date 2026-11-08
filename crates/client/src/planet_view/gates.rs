//! The planet's World Gates on show: rings on its picture once the colours are
//! done, and a line in the bar on how they were found.

use bevy::prelude::*;
use shared::world_gate::{WORLD_GATES, WorldGates};

use super::showing::PlanetView;

/// A gate's ring on the picture: its radius in pixels, its thickness, and
/// its colour (bright, as nothing on a planet is this colour).
pub(super) const RING: f32 = 5.0;
const RING_WIDTH: f32 = 1.6;
const RING_COLOUR: [u8; 4] = [255, 40, 220, 255];

/// Draws a ring round each gate into the picture (`size` pixels, of a planet
/// `width` grid points round), over whatever is there.
pub(super) fn stamp_gates(gates: &WorldGates, size: UVec2, width: i64, data: &mut [u8]) {
    for gate in &gates.gates {
        let column = gate.x as f32 / width as f32 * size.x as f32;
        let row = (gate.y as f32 / (width as f32 / 2.0) + 0.5) * size.y as f32;
        let reach = (RING + RING_WIDTH).ceil() as i32;
        for down in -reach..=reach {
            for across in -reach..=reach {
                let distance = ((across * across + down * down) as f32).sqrt();
                if (distance - RING).abs() > RING_WIDTH / 2.0 {
                    continue;
                }
                let x = (column as i32 + across).rem_euclid(size.x as i32) as u32;
                let y = row as i32 + down;
                if y < 0 || y >= size.y as i32 {
                    continue;
                }
                let start = ((y as u32 * size.x + x) * 4) as usize;
                data[start..start + 4].copy_from_slice(&RING_COLOUR);
            }
        }
    }
}

/// How the gates were found, for the bar.
pub(super) fn gate_words(gates: &WorldGates) -> String {
    let found = gates.gates.len();
    if found < WORLD_GATES {
        return format!(
            "Only {found} of {WORLD_GATES} World Gates, {} tries",
            gates.tries
        );
    }
    let round = gates.gates.iter().map(|gate| gate.round).max().unwrap_or(0) + 1;
    format!("{found} World Gates, round {round}, {} tries", gates.tries)
}

/// Keeps the bar's line about the gates up to date with the globe.
pub(super) fn show_gate_words(view: Res<PlanetView>, mut texts: Query<&mut Text>) {
    let Some(showing) = view.showing.as_ref() else {
        return;
    };
    let words = gate_words(&showing.globe.planet.gates);
    if let Ok(mut text) = texts.get_mut(showing.bar.gate_words)
        && text.0 != words
    {
        text.0 = words;
    }
}
