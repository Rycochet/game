//! Others on the map: mobs and other players, as tokens, for as long as the server
//! says they can be seen. See `docs/context/client/entities.md`.

use std::collections::HashMap;

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::sheet::Feeling;

use crate::common::{
    Attitude, DetectMonsters, GameView, Icon, MapPosition, Other, SmoothPosition, Token, TokenSize,
};
use crate::connection::{FromServer, ToServer};
use crate::inventory::Carried;
use crate::player::{Player, PlayerSystems};

/// How fast others are shown moving, in grid units a second: the same as an
/// ordinary player, so they keep up with what the server says.
const WORLD_MAP_SPEED: f64 = 60.0;
const SITE_SPEED: f64 = 12.0;
/// Further behind than this, in grid units, they go straight there.
const TOO_FAR_BEHIND: f64 = 8.0;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Others>().add_systems(
            Update,
            (read_server, show_moving, show_own_kind)
                .chain()
                .after(PlayerSystems::Travel),
        );
    }
}

/// The picture for a kind of creature.
fn picture(kind: CreatureKind) -> &'static str {
    match kind {
        // (No centaur among the pictures yet.)
        CreatureKind::Centaur | CreatureKind::Human => "monsters/humanfighter.png",
        CreatureKind::Beggar => "monsters/humandruid01.png",
        CreatureKind::Chicken => "monsters/chicken1.png",
        CreatureKind::Giant => "monsters/gianthill.png",
        CreatureKind::Horse => "monsters/horse01.png",
        CreatureKind::Kobold => "monsters/koboldfighter.png",
    }
}

/// The colour of a token, from how what it stands for feels about the player.
fn attitude(feeling: Feeling) -> Attitude {
    match feeling {
        Feeling::None => Attitude::None,
        Feeling::Friendly => Attitude::Friendly,
        Feeling::Neutral => Attitude::Neutral,
        Feeling::Unfriendly => Attitude::Unfriendly,
    }
}

/// How big a kind of creature's picture is drawn.
fn picture_size(kind: CreatureKind) -> TokenSize {
    match kind {
        CreatureKind::Chicken => TokenSize::Small,
        CreatureKind::Giant => TokenSize::Large,
        _ => TokenSize::Default,
    }
}

/// Others on the map, by their ids.
#[derive(Resource, Default)]
struct Others(HashMap<Id, Entity>);

fn read_server(
    mut from_server: MessageReader<FromServer>,
    mut others: ResMut<Others>,
    mut positions: Query<&mut MapPosition, With<Other>>,
    detect: Res<DetectMonsters>,
    mut to_server: MessageWriter<ToServer>,
    mut commands: Commands,
) {
    for FromServer(message) in from_server.read() {
        match message {
            ServerMessage::EntitySeen(look) => {
                let at = MapPosition {
                    x: look.x,
                    y: look.y,
                };
                let spawned = commands
                    .spawn((
                        Other {
                            id: look.id,
                            kind: look.kind,
                            feeling: look.feeling,
                        },
                        at,
                        SmoothPosition::from(at),
                        Token {
                            size: picture_size(look.kind),
                            attitude: attitude(look.feeling),
                            ..default()
                        },
                        Icon(picture(look.kind).into()),
                    ))
                    .id();
                if let Some(old) = others.0.insert(look.id, spawned) {
                    commands.entity(old).despawn();
                }
            }
            ServerMessage::EntityMoved { id, x, y } => {
                if let Some(&entity) = others.0.get(id)
                    && let Ok(mut position) = positions.get_mut(entity)
                {
                    *position = MapPosition { x: *x, y: *y };
                }
            }
            ServerMessage::EntityGone { id } => {
                if let Some(entity) = others.0.remove(id) {
                    commands.entity(entity).despawn();
                }
            }
            // A new connection knows nothing of this.
            ServerMessage::Welcome { .. } if detect.0 => {
                to_server.write(ToServer(ClientMessage::DevDetectMonsters(true)));
                for (_, entity) in others.0.drain() {
                    commands.entity(entity).despawn();
                }
            }
            // Another map, or a new connection: the server says who is there.
            ServerMessage::Welcome { .. }
            | ServerMessage::EnterLevel(_)
            | ServerMessage::OnWorldMap { .. }
            | ServerMessage::Teleported { .. } => {
                for (_, entity) in others.0.drain() {
                    commands.entity(entity).despawn();
                }
            }
            _ => {}
        }
    }
}

/// `distance` towards `to`, stopping exactly on it.
fn glide(from: Vec2d, to: Vec2d, distance: f64) -> Vec2d {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let gap = dx.hypot(dy);
    if gap <= distance || gap > TOO_FAR_BEHIND {
        to
    } else {
        (from.0 + dx / gap * distance, from.1 + dy / gap * distance)
    }
}

type Vec2d = (f64, f64);

/// Eases others towards where they are, between the server's word.
fn show_moving(
    time: Res<Time>,
    view: Res<State<GameView>>,
    mut others: Query<(&MapPosition, &mut SmoothPosition), With<Other>>,
) {
    let speed = match view.get() {
        GameView::Dungeon => SITE_SPEED,
        _ => WORLD_MAP_SPEED,
    };
    let distance = speed * time.delta_secs_f64();
    for (at, mut smooth) in &mut others {
        let to = (at.x as f64, at.y as f64);
        let (x, y) = glide((smooth.x, smooth.y), to, distance);
        if (x, y) != (smooth.x, smooth.y) {
            *smooth = SmoothPosition { x, y };
        }
    }
}

/// The player's own token shows what kind of creature they are.
fn show_own_kind(
    carried: Res<Carried>,
    mut drawn: Local<Option<CreatureKind>>,
    player: Single<(&mut Icon, &mut Token), With<Player>>,
) {
    let kind = carried.0.creature;
    if *drawn == Some(kind) {
        return;
    }
    let first = drawn.is_none();
    *drawn = Some(kind);
    // A human keeps the picture they start with.
    if first && kind == CreatureKind::Human {
        return;
    }
    let (mut icon, mut token) = player.into_inner();
    icon.0 = match kind {
        CreatureKind::Human => "monsters/humanarcher.png".into(),
        other => picture(other).into(),
    };
    token.size = picture_size(kind);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn others_glide_and_jump_only_when_far_behind() {
        assert_eq!(glide((0.0, 0.0), (1.0, 0.0), 0.25), (0.25, 0.0));
        assert_eq!(glide((0.0, 0.0), (1.0, 0.0), 2.0), (1.0, 0.0));
        assert_eq!(glide((0.0, 0.0), (50.0, 0.0), 0.25), (50.0, 0.0));
    }

    #[test]
    fn every_kind_has_a_picture_that_is_there() {
        for kind in CreatureKind::ALL {
            let file = format!("{}/assets/{}", env!("CARGO_MANIFEST_DIR"), picture(kind));
            assert!(std::path::Path::new(&file).exists(), "{file}");
        }
    }
}
