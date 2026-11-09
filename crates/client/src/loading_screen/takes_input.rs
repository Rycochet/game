//! When the loading screen takes all input: while it is up after Play, but not
//! at the startup menu, which takes what it wants, nor while choosing or making a
//! character or logging in, whose screens take it.

use bevy::prelude::*;

use crate::common::{CharacterSelect, LoadingStep, LoggingIn, MakingCharacter, Session};

pub(super) fn takes_input(
    step: Res<LoadingStep>,
    session: Res<Session>,
    making: Res<MakingCharacter>,
    select: Res<CharacterSelect>,
    logging_in: Res<LoggingIn>,
) -> bool {
    *step != LoadingStep::Ready
        && session.is_playing()
        && !making.0
        && !select.open
        && !logging_in.shows()
}
