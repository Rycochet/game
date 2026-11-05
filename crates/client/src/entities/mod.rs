//! Others on the map: mobs and other players, as tokens, for as long as the server
//! says they can be seen. See `docs/context/client/entities.md`.

mod dressed;

use std::collections::HashMap;

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::sheet::Feeling;

use crate::common::{
    Attitude, Clickable, DetectMonsters, GameView, Icon, KillMob, MapPosition, Other,
    SmoothPosition, Token, TokenSize,
};
use crate::connection::{FromServer, ToServer};
use crate::inventory::Carried;
use crate::player::{Player, PlayerSystems};
use crate::view3d::{ClickedToken, View3dSystems};

/// How fast others are shown moving, in grid units a second: the same as an
/// ordinary player, so they keep up with what the server says.
const WORLD_MAP_SPEED: f64 = 60.0;
const SITE_SPEED: f64 = shared::energy::SITE_CELLS_PER_SECOND;
/// Further behind than this, in grid units, they go straight there.
const TOO_FAR_BEHIND: f64 = 8.0;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Others>().add_systems(
            Update,
            (
                (
                    read_server,
                    show_moving,
                    show_own_kind,
                    dressed::show_own_dress,
                )
                    .chain()
                    .after(PlayerSystems::Travel),
                kill_clicked.after(View3dSystems::Click),
                offer_to_kill.after(read_server),
            ),
        );
    }
}

/// The picture for a kind of creature.
fn picture(kind: CreatureKind) -> &'static str {
    match kind {
        // (No centaur, dwarf, light elf, goblin, minotaur or birdfolk among the pictures
        // yet.)
        CreatureKind::Centaur
        | CreatureKind::Human
        | CreatureKind::Dwarf
        | CreatureKind::Elf
        | CreatureKind::Goblin
        | CreatureKind::Minotaur
        | CreatureKind::Birdfolk => "monsters/humanfighter.png",
        CreatureKind::Beggar => "monsters/humandruid01.png",
        CreatureKind::Chicken => "monsters/chicken1.png",
        CreatureKind::Giant => "monsters/gianthill.png",
        CreatureKind::Horse => "monsters/horse01.png",
        CreatureKind::Kobold => "monsters/koboldfighter.png",
        CreatureKind::Halfling => "monsters/halflingfighter.png",
        CreatureKind::Lizardfolk => "monsters/lizardgreen.png",
        CreatureKind::Orc => "monsters/orcfighter.png",
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
        CreatureKind::Giant | CreatureKind::Minotaur => TokenSize::Large,
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
    mut drawing: dressed::Drawing,
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
                if let Some(picture) = drawing.dressed(look.kind, &look.appearance, &look.worn) {
                    commands.entity(spawned).insert(picture);
                }
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
            | ServerMessage::EnterHall { .. }
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

/// With Kill Mob on, mobs' tokens can be clicked, which also stops the click walking
/// there. Other players feel nothing, so stay as they are.
fn offer_to_kill(
    kill: Res<KillMob>,
    others: Query<(Entity, &Other, Has<Clickable>)>,
    mut commands: Commands,
) {
    for (entity, other, clickable) in &others {
        let wanted = kill.0 && other.feeling != Feeling::None;
        if wanted && !clickable {
            commands.entity(entity).insert(Clickable);
        } else if clickable && !wanted {
            commands.entity(entity).remove::<Clickable>();
        }
    }
}

/// A click or tap on a mob's token with Kill Mob on. It goes when the server says.
fn kill_clicked(
    kill: Res<KillMob>,
    clicked: Res<ClickedToken>,
    others: Query<&Other>,
    mut to_server: MessageWriter<ToServer>,
) {
    if kill.0
        && let Some(other) = clicked.0.and_then(|entity| others.get(entity).ok())
    {
        to_server.write(ToServer(ClientMessage::DevKillMob { mob: other.id }));
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
mod tests;
