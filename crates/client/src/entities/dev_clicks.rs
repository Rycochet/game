//! Dev tools' clicks on mobs: Kill Mob, Inspect Mob and Delete Mob make their
//! tokens clickable, and a click asks the server to do that to the mob.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, DevMobTool};
use shared::sheet::Feeling;

use crate::common::{Clickable, DeleteMob, InspectMob, KillMob, Other};
use crate::connection::ToServer;
use crate::view3d::ClickedToken;

/// With one of them on, mobs' tokens can be clicked, which also stops the click
/// walking there. Other players feel nothing, so stay as they are.
pub(super) fn offer_mobs(
    kill: Res<KillMob>,
    inspect: Res<InspectMob>,
    delete: Res<DeleteMob>,
    others: Query<(Entity, &Other, Has<Clickable>)>,
    mut commands: Commands,
) {
    let on = kill.0 || inspect.0 || delete.0;
    for (entity, other, clickable) in &others {
        let wanted = on && other.feeling != Feeling::None;
        if wanted && !clickable {
            commands.entity(entity).insert(Clickable);
        } else if clickable && !wanted {
            commands.entity(entity).remove::<Clickable>();
        }
    }
}

/// A click or tap on a mob's token with one of them on. A killed or deleted mob
/// goes when the server says; what Inspect Mob asks comes back as `DevMobInfo`.
pub(super) fn mob_clicked(
    kill: Res<KillMob>,
    inspect: Res<InspectMob>,
    delete: Res<DeleteMob>,
    clicked: Res<ClickedToken>,
    others: Query<&Other>,
    mut to_server: MessageWriter<ToServer>,
) {
    let Some(other) = clicked.0.and_then(|entity| others.get(entity).ok()) else {
        return;
    };
    let mob = other.id;
    let message = if kill.0 {
        ClientMessage::DevKillMob { mob }
    } else if inspect.0 {
        ClientMessage::DevMobs(DevMobTool::Inspect(mob))
    } else if delete.0 {
        ClientMessage::DevMobs(DevMobTool::Delete(mob))
    } else {
        return;
    };
    to_server.write(ToServer(message));
}
