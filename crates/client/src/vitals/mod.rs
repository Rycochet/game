//! The player's Health and Mana: a circle at the bottom middle of the window,
//! red Health on the left and blue Mana on the right, filling from the bottom.
//! See `docs/context/client/vitals.md`.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use shared::protocol::ServerMessage;

use crate::common::{BottomHud, GameView, OwnVitals, ToldVitals, VitalsCircle, interface_area};
use crate::connection::FromServer;
use crate::widgets::{HoverTip, UiId};

mod paint;

/// The circle across, as a share of the window's height: half the minimap's
/// (owner, 2026-11-06).
const ACROSS_OF_HEIGHT: f32 = 0.1;
/// The gap under it, in interface pixels.
const BOTTOM_GAP: f32 = 8.0;

pub struct VitalsPlugin;

impl Plugin for VitalsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_circle)
            .add_systems(Update, (take_vitals, show_circle).chain());
    }
}

/// The circle's node.
#[derive(Component)]
struct Circle;

/// Its picture, and what it was last painted with: its side in screen pixels
/// and how full each half is, in steps finer than a pixel.
#[derive(Resource)]
struct CirclePicture {
    image: Handle<Image>,
    painted: Option<(u32, u32, u32)>,
}

fn spawn_circle(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = images.add(Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    commands.spawn((
        Circle,
        UiId::new("hud", "vitals"),
        ImageNode::new(image.clone()),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(BOTTOM_GAP),
            ..default()
        },
        HoverTip(String::new()),
        Visibility::Hidden,
    ));
    commands.insert_resource(CirclePicture {
        image,
        painted: None,
    });
}

/// Keeps what the server says of the player's own Health and Mana.
fn take_vitals(
    mut from_server: MessageReader<FromServer>,
    time: Res<Time>,
    mut own: ResMut<OwnVitals>,
) {
    for FromServer(message) in from_server.read() {
        if let ServerMessage::Vitals {
            who: None,
            health,
            mana,
        } = message
        {
            own.0 = Some(ToldVitals {
                health: *health,
                mana: *mana,
                at: time.elapsed_secs_f64(),
            });
        }
    }
}

/// Places and sizes the circle, paints it when how full it is changes by
/// a part of a pixel, and says what it holds when hovered.
#[allow(clippy::too_many_arguments)]
fn show_circle(
    view: Res<State<GameView>>,
    own: Res<OwnVitals>,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    mut picture: ResMut<CirclePicture>,
    mut images: ResMut<Assets<Image>>,
    mut circle: Single<(&mut Node, &mut Visibility, &mut HoverTip), With<Circle>>,
    mut bottom: ResMut<BottomHud>,
    mut placed: ResMut<VitalsCircle>,
) {
    let (node, visibility, tip) = &mut *circle;
    let told = own.0.filter(|_| *view.get() != GameView::Connecting);
    let Some(told) = told else {
        visibility.set_if_neq(Visibility::Hidden);
        bottom.set_if_neq(BottomHud(0.0));
        placed.set_if_neq(VitalsCircle::default());
        return;
    };
    visibility.set_if_neq(Visibility::Inherited);
    let room = interface_area(&window, &ui_scale);
    let across = (room.y * ACROSS_OF_HEIGHT).round().max(1.0);
    node.width = px(across);
    node.height = px(across);
    node.left = px(((room.x - across) / 2.0).round());
    bottom.set_if_neq(BottomHud(BOTTOM_GAP + across));
    placed.set_if_neq(VitalsCircle {
        across,
        bottom: BOTTOM_GAP,
    });
    let (health, mana) = told.now(time.elapsed_secs_f64());
    let words = format!(
        "Health {} / {}\nMana {} / {}",
        health.ceil() as u32,
        told.health.most,
        mana.ceil() as u32,
        told.mana.most
    );
    if tip.0 != words {
        tip.0 = words;
    }
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
