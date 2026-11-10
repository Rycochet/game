//! Web: whether the server the page came from is there to play on
//! (`common::HomeServer`), asked once at start. A page hosted without a game
//! server behind it (such as `trunk serve`) can only Connect (owner,
//! 2026-11-10).

use bevy::prelude::*;
use std::sync::{Arc, Mutex};

use crate::common::HomeServer;

/// The answer, once the browser has it.
#[derive(Resource)]
pub(super) struct HomeAnswer(Arc<Mutex<Option<bool>>>);

/// Asks the page's server for its version.
pub(super) fn ask_home(mut commands: Commands) {
    let answer = Arc::new(Mutex::new(None));
    let kept = answer.clone();
    super::probe::ask_page_server(move |found| {
        *kept.lock().expect("never poisoned") = Some(found);
    });
    commands.insert_resource(HomeAnswer(answer));
}

/// Puts the answer in `HomeServer` once it has come.
pub(super) fn take_home_answer(
    mut commands: Commands,
    answer: Option<Res<HomeAnswer>>,
    mut home: ResMut<HomeServer>,
) {
    let Some(answer) = answer else {
        return;
    };
    let Some(found) = *answer.0.lock().expect("never poisoned") else {
        return;
    };
    if found {
        *home = HomeServer::Found;
    } else {
        info!("No game server where this page came from: only Connect can play");
        *home = HomeServer::Missing;
    }
    commands.remove_resource::<HomeAnswer>();
}
