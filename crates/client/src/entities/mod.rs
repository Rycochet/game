//! Others on the map: mobs and other players, as tokens, for as long as the server
//! says they can be seen. See `docs/context/client/entities.md`.

mod boss_look;
mod dressed;
mod dressing_wait;
mod facing;
mod flash;
mod health_bars;
mod hover_name;
mod morph;
mod moving;
mod pictures;
mod wind_up_bars;

use pictures::{picture, picture_size};

use std::collections::HashMap;

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::sheet::Feeling;

use crate::common::{Attitude, DetectMonsters, Icon, MapPosition, Other, SpotPosition, Token};
use crate::connection::{FromServer, ToServer};
use crate::inventory::Carried;
use crate::player::{Player, PlayerSystems};

use moving::{Goal, show_moving};

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Others>()
            .init_resource::<health_bars::SeenHealth>()
            .init_resource::<wind_up_bars::SeenWindUps>()
            .init_resource::<dressing_wait::DressingWait>()
            .add_message::<crate::common::Dressing>()
            .add_systems(
                PostUpdate,
                (
                    health_bars::take_health,
                    health_bars::show_health_bars,
                    wind_up_bars::take_wind_ups,
                    dressing_wait::take_dressing,
                    wind_up_bars::show_wind_up_bars,
                )
                    .chain()
                    .after(TransformSystems::Propagate),
            )
            .add_systems(
                Update,
                (
                    (
                        read_server,
                        show_moving,
                        show_own_kind,
                        dressed::show_own_dress,
                        morph::play_morphs,
                    )
                        .chain()
                        .after(PlayerSystems::Travel),
                    flash::flash_harmed.after(read_server),
                    facing::read_turns.after(read_server),
                    boss_look::show_bosses.after(read_server),
                ),
            );
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

/// Others on the map, by their ids.
#[derive(Resource, Default)]
struct Others(HashMap<Id, Entity>);

#[allow(clippy::too_many_arguments)]
fn read_server(
    mut from_server: MessageReader<FromServer>,
    mut others: ResMut<Others>,
    mut positions: Query<(&mut MapPosition, &mut SpotPosition, &mut Goal), With<Other>>,
    mut on_level: Local<bool>,
    seen: Query<(&Other, &morph::SeenLooks)>,
    time: Res<Time>,
    detect: Res<DetectMonsters>,
    mut to_server: MessageWriter<ToServer>,
    mut drawing: dressed::Drawing,
    mut commands: Commands,
) {
    for FromServer(message) in from_server.read() {
        match message {
            ServerMessage::EntitySeen(look) => {
                let (at, smooth) = moving::placed(*on_level, (look.x, look.y));
                let spawned = commands
                    .spawn((
                        Other {
                            id: look.id,
                            kind: look.kind,
                            feeling: look.feeling,
                        },
                        at,
                        // Where the server has it (units on a level).
                        SpotPosition(shared::spot::Spot::new(look.x, look.y)),
                        crate::common::Facing(look.heading),
                        smooth,
                        Goal::still(smooth),
                        Token {
                            size: picture_size(look.kind),
                            base: look.kind.def().size,
                            attitude: attitude(look.feeling),
                            ..default()
                        },
                        Icon(picture(look.kind).into()),
                        morph::SeenLooks(look.appearance.clone()),
                        hover_name::hover_name(look),
                    ))
                    .id();
                if let Some(mark) = &look.mark {
                    commands
                        .entity(spawned)
                        .insert(crate::common::StaffMark(mark.shown().to_owned()));
                }
                if let Some(boss) = &look.boss {
                    commands
                        .entity(spawned)
                        .insert(boss_look::SeenBoss(boss.clone()));
                }
                let picture = drawing.dressed(look.kind, &look.appearance, &look.worn);
                // Seen again as another kind: it changes into it.
                let old = others.0.insert(look.id, spawned);
                let was = old.and_then(|old| seen.get(old).ok());
                let morph = was.filter(|(other, _)| other.kind != look.kind).and_then(
                    |(other, morph::SeenLooks(appearance))| {
                        let from = (other.kind, appearance);
                        let to = (look.kind, &look.appearance);
                        drawing.morph(from, to, picture.clone(), time.elapsed_secs_f64())
                    },
                );
                match morph {
                    Some(morph) => {
                        commands.entity(spawned).insert(morph);
                    }
                    None => dressed::show(&mut commands, spawned, picture),
                }
                if let Some(old) = old {
                    commands.entity(old).despawn();
                }
            }
            ServerMessage::Moves(stretches) => {
                for stretch in stretches {
                    if let Some(&entity) = others.0.get(&stretch.id)
                        && let Ok((mut position, mut spot, mut goal)) = positions.get_mut(entity)
                    {
                        (*position, *goal) = moving::stretch(*on_level, stretch);
                        spot.0 = shared::spot::Spot::new(stretch.to.0, stretch.to.1);
                    }
                }
            }
            ServerMessage::EntityGone { id, .. } => {
                if let Some(entity) = others.0.remove(id) {
                    commands.entity(entity).despawn();
                }
            }
            // A new connection knows nothing of this.
            ServerMessage::Welcome { .. } if detect.0 => {
                *on_level = false;
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
                *on_level = match message {
                    ServerMessage::EnterLevel(_) | ServerMessage::EnterHall { .. } => true,
                    ServerMessage::Teleported { .. } => *on_level,
                    _ => false,
                };
                for (_, entity) in others.0.drain() {
                    commands.entity(entity).despawn();
                }
            }
            _ => {}
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
