//! The loading screen: the game's name and a bar saying what is happening,
//! covering everything from the first frame until the game can take input, and
//! again while the connection is lost. See `docs/context/client/loading_screen.md`.

use bevy::diagnostic::FrameCount;
use bevy::input::InputSystems;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::common::{
    GameView, LoadingStep, MakingCharacter, PointerSystems, ServerStatus, Session, TouchGesture,
    interface_area,
};

mod frames;
use crate::widgets::{Progress, TextSize, WidgetSystems, layer_z, spawn_progress_bar};
use frames::keep_frame_rate;

/// Built into the program, rather than loaded, to be there on the first frame.
const TITLE_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/CinzelDecorative-Bold.ttf");

const TITLE: &str = "Hall of\nWorlds";
/// The share of the window the title may fill.
const TITLE_HEIGHT_SHARE: f32 = 0.5;
/// At the startup menu, the title keeps to the top of the window, above the menu.
const START_TITLE_HEIGHT_SHARE: f32 = 0.28;
const START_TITLE_TOP: f32 = 6.0;
const TITLE_WIDTH_SHARE: f32 = 0.8;
/// How wide the title's longest line is ("Worlds" in this font), and how tall
/// its two lines are, in font sizes.
const TITLE_EMS_WIDE: f32 = 4.61;
const TITLE_EMS_HIGH: f32 = 2.4;

const BACKGROUND: Color = Color::srgb(0.04, 0.03, 0.07);
const GOLD: Color = Color::srgb(0.93, 0.75, 0.38);
const TITLE_SHADOW: Color = Color::srgb(0.35, 0.12, 0.05);
const BAR_BACKGROUND: Color = Color::srgb(0.13, 0.11, 0.16);
const WRITING: Color = Color::srgb(0.95, 0.92, 0.85);
const REFUSED: Color = Color::srgb(1.0, 0.35, 0.3);

const BAR_HEIGHT: f32 = 32.0;
const BAR_MARGIN: f32 = 32.0;
const BAR_FONT: TextSize = TextSize::Large;

/// Frames before the screen has certainly been drawn once, so "Starting..." is
/// seen before the work starts.
const FRAMES_TO_DRAW: u32 = 3;
/// Frames after being welcomed, for the view to be built and drawn and the
/// messages following the welcome (such as being in a site) to arrive.
const FRAMES_TO_SETTLE: u32 = 3;

pub struct LoadingScreenPlugin;

impl Plugin for LoadingScreenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HoldLoadingScreen>()
            .add_systems(Startup, spawn_screen)
            .add_systems(
                PreUpdate,
                // At the startup menu, the menu over the screen takes what it
                // wants, as making a character does.
                take_all_input
                    .run_if(
                        |step: Res<LoadingStep>,
                         session: Res<Session>,
                         making: Res<MakingCharacter>| {
                            *step != LoadingStep::Ready && session.is_playing() && !making.0
                        },
                    )
                    .after(InputSystems)
                    .after(PointerSystems)
                    .before(WidgetSystems::Presses),
            )
            .add_systems(
                Update,
                (update_step, show_screen, fit_title, keep_frame_rate).chain(),
            );

        #[cfg(target_arch = "wasm32")]
        app.add_systems(Update, remove_page_screen.after(update_step));
    }
}

/// Keeps the screen up whatever happens (dev tools), to look at it.
#[derive(Resource, Default)]
pub struct HoldLoadingScreen(pub bool);

#[derive(Component)]
struct LoadingScreen;

#[derive(Component)]
struct Title;

#[derive(Component)]
struct Bar;

#[derive(Component)]
struct BarText;

/// The next step: `frames` is how many frames have been run, `settled` how many
/// of them in a row the client has spent welcomed.
fn next_step(step: LoadingStep, frames: u32, settled: u32) -> LoadingStep {
    if step == LoadingStep::Starting && frames < FRAMES_TO_DRAW {
        LoadingStep::Starting
    } else if settled > FRAMES_TO_SETTLE {
        LoadingStep::Ready
    } else {
        LoadingStep::Connecting
    }
}

/// The biggest font size that keeps the title in its share of the window, which
/// is less at the startup menu.
fn title_font_size(window: Vec2, at_start: bool) -> f32 {
    let height_share = if at_start {
        START_TITLE_HEIGHT_SHARE
    } else {
        TITLE_HEIGHT_SHARE
    };
    (window.x * TITLE_WIDTH_SHARE / TITLE_EMS_WIDE).min(window.y * height_share / TITLE_EMS_HIGH)
}

/// What the bar says, and how full it is.
fn bar_shows(step: LoadingStep, status: &ServerStatus) -> (String, f32) {
    if let ServerStatus::Refused(reason) = status {
        return (reason.clone(), 0.0);
    }
    match step {
        LoadingStep::Starting => ("Starting...".into(), 0.0),
        LoadingStep::Connecting | LoadingStep::Ready => ("Connecting...".into(), 0.5),
    }
}

fn spawn_screen(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = fonts.add(Font::from_bytes(TITLE_FONT_BYTES.to_vec()));
    commands
        .spawn((
            LoadingScreen,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(BACKGROUND),
            // Above everything but the full-screen layers, so the system menu
            // can be the startup menu over it.
            GlobalZIndex(layer_z(0) - 1),
        ))
        .with_children(|screen| {
            screen.spawn((
                Title,
                Text::new(TITLE),
                TextFont {
                    font: font.into(),
                    ..default()
                },
                TextLayout::justify(Justify::Center),
                TextColor(GOLD),
                TextShadow {
                    offset: Vec2::splat(3.0),
                    color: TITLE_SHADOW,
                },
            ));
            spawn_progress_bar(
                screen,
                0.0,
                (BAR_BACKGROUND, GOLD.with_alpha(0.45)),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(BAR_MARGIN),
                    right: px(BAR_MARGIN),
                    bottom: px(BAR_MARGIN),
                    height: px(BAR_HEIGHT),
                    ..default()
                },
                (BarText, Text::new(""), BAR_FONT.font(), TextColor(WRITING)),
            )
            .insert(Bar);
        });
}

/// Nothing behind the screen gets any input: no keys, clicks, wheel or fingers.
fn take_all_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut scroll: ResMut<AccumulatedMouseScroll>,
    mut gesture: ResMut<TouchGesture>,
) {
    keys.reset_all();
    mouse.reset_all();
    *scroll = default();
    *gesture = default();
}

fn update_step(
    frames: Res<FrameCount>,
    status: Res<ServerStatus>,
    view: Res<State<GameView>>,
    hold: Res<HoldLoadingScreen>,
    mut step: ResMut<LoadingStep>,
    mut settled: Local<u32>,
) {
    let welcomed = status.is_connected() && *view.get() != GameView::Connecting && !hold.0;
    *settled = if welcomed { *settled + 1 } else { 0 };
    step.set_if_neq(next_step(*step, frames.0, *settled));
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn show_screen(
    step: Res<LoadingStep>,
    status: Res<ServerStatus>,
    session: Res<Session>,
    making: Res<MakingCharacter>,
    mut screen: Single<&mut Visibility, (With<LoadingScreen>, Without<Bar>)>,
    mut bar: Single<&mut Visibility, (With<Bar>, Without<LoadingScreen>)>,
    mut progress: Single<&mut Progress, With<Bar>>,
    mut text: Single<(&mut Text, &mut TextColor), With<BarText>>,
) {
    if !step.is_changed() && !status.is_changed() && !session.is_changed() && !making.is_changed() {
        return;
    }
    let shown_if = |shown: bool| {
        if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };
    screen.set_if_neq(shown_if(*step != LoadingStep::Ready));
    // Only the title behind the startup menu, or the screen making a character.
    bar.set_if_neq(shown_if(session.is_playing() && !making.0));
    let (words, full) = bar_shows(*step, &status);
    progress.0 = full;
    text.0.0 = words;
    text.1.0 = if matches!(*status, ServerStatus::Refused(_)) {
        REFUSED
    } else {
        WRITING
    };
}

fn fit_title(
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    session: Res<Session>,
    mut title: Single<&mut TextFont, With<Title>>,
    mut screen: Single<&mut Node, With<LoadingScreen>>,
    mut last: Local<Option<(Vec2, bool)>>,
) {
    // The title keeps its share of the window, whatever size the interface is.
    let size = interface_area(&window, &ui_scale);
    let at_start = !session.is_playing();
    // Minimised: nothing to fit, and no size of writing would.
    if *last != Some((size, at_start)) && size.min_element() >= 1.0 {
        *last = Some((size, at_start));
        title.font_size = title_font_size(size, at_start).into();
        (screen.justify_content, screen.padding) = if at_start {
            (
                JustifyContent::FlexStart,
                UiRect::top(Val::Vh(START_TITLE_TOP)),
            )
        } else {
            (JustifyContent::Center, UiRect::ZERO)
        };
    }
}

/// Removes the screen in `index.html`, shown during the download, once this one
/// has certainly been drawn to take its place.
#[cfg(target_arch = "wasm32")]
fn remove_page_screen(step: Res<LoadingStep>, mut removed: Local<bool>) {
    if *removed || *step == LoadingStep::Starting {
        return;
    }
    *removed = true;
    let screen = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("loading"));
    if let Some(screen) = screen {
        screen.remove();
    }
}

#[cfg(test)]
mod tests;
