//! Keeping the circle as the player's Health and Mana are.

use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::window::PrimaryWindow;

use super::{ACROSS_OF_HEIGHT, BALL, BOTTOM_GAP, CirclePicture, paint};
use crate::common::{BottomHud, GameView, OwnVitals, VitalsCircle, interface_area};
use crate::dom::Dom;

/// What the circle's element was last told.
#[derive(Default, PartialEq)]
pub(super) struct Told {
    shown: bool,
    across: f32,
    left: f32,
    tip: String,
}

/// Places and sizes the circle, paints it when how full it is changes by
/// a part of a pixel, and says what it holds when hovered.
#[allow(clippy::too_many_arguments)]
pub(super) fn show_circle(
    view: Res<State<GameView>>,
    own: Res<OwnVitals>,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut picture: ResMut<CirclePicture>,
    mut images: ResMut<Assets<Image>>,
    (mut bottom, mut placed): (ResMut<BottomHud>, ResMut<VitalsCircle>),
    mut dom: Dom,
    mut was: Local<Told>,
) {
    let told = own.0.filter(|_| *view.get() != GameView::Connecting);
    let Some(told) = told else {
        if was.shown {
            dom.set_flag(BALL, "hidden", true);
            was.shown = false;
        }
        bottom.set_if_neq(BottomHud(0.0));
        placed.set_if_neq(VitalsCircle::default());
        return;
    };
    let room = interface_area(&window, &ui_scale);
    let across = (room.y * ACROSS_OF_HEIGHT).round().max(1.0);
    bottom.set_if_neq(BottomHud(BOTTOM_GAP + across));
    placed.set_if_neq(VitalsCircle {
        across,
        bottom: BOTTOM_GAP,
    });
    let (health, mana) = told.now(time.elapsed_secs_f64());
    let now = Told {
        shown: true,
        across,
        left: ((room.x - across) / 2.0).round(),
        tip: format!(
            "Health {} / {}\nMana {} / {}",
            health.ceil() as u32,
            told.health.most,
            mana.ceil() as u32,
            told.mana.most
        ),
    };
    tell(&mut dom, &was, &now);
    *was = now;
    // Painted in the screen's pixels, so it stays sharp.
    let side = (across * window.scale_factor() * ui_scale.0)
        .round()
        .max(1.0) as u32;
    let share = |value: f32, most: u32| value / most.max(1) as f32;
    let (health, mana) = (share(health, told.health.most), share(mana, told.mana.most));
    // Repainted when either level moves a quarter of a pixel.
    let steps = |share: f32| (share * side as f32 * 4.0).round() as u32;
    let painted = Some((side, steps(health), steps(mana)));
    if picture.painted == painted {
        return;
    }
    picture.painted = painted;
    let Some(mut image) = images.get_mut(&picture.image) else {
        return;
    };
    image.resize(Extent3d {
        width: side,
        height: side,
        depth_or_array_layers: 1,
    });
    image.data = Some(paint::paint(side, health, mana));
}

/// Changes the circle's element, only where it differs from what it was told.
fn tell(dom: &mut Dom, was: &Told, now: &Told) {
    if !was.shown {
        dom.set_flag(BALL, "hidden", false);
    }
    if was.across != now.across {
        dom.set(BALL, "width", &now.across.to_string());
        dom.set(BALL, "height", &now.across.to_string());
    }
    if was.left != now.left {
        dom.set(BALL, "left", &now.left.to_string());
    }
    if was.tip != now.tip {
        dom.set(BALL, "title", &now.tip);
    }
}
