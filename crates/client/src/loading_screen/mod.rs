//! The loading screen: the game's name and a bar saying what is happening,
//! covering everything from the first frame until the game can take input, and
//! again while the connection is lost. See `docs/context/client/loading_screen.md`.

use bevy::diagnostic::FrameCount;
use bevy::input::InputSystems;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitSettings;

use crate::common::{
    GameView, LoadingStep, PointerSystems, ServerStatus, TouchGesture, frame_settings,
};
use crate::widgets::WidgetSystems;

/// Built into the program, rather than loaded, to be there on the first frame.
const TITLE_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/CinzelDecorative-Bold.ttf");

const TITLE: &str = "Hall of\nWorlds";
/// The share of the window the title may fill.
const TITLE_HEIGHT_SHARE: f32 = 0.5;
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
const BAR_FONT: f32 = 18.0;

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
                take_all_input
                    .run_if(|step: Res<LoadingStep>| *step != LoadingStep::Ready)
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
struct BarFill;

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

/// The biggest font size that keeps the title in its share of the window.
fn title_font_size(window: Vec2) -> f32 {
    (window.x * TITLE_WIDTH_SHARE / TITLE_EMS_WIDE)
        .min(window.y * TITLE_HEIGHT_SHARE / TITLE_EMS_HIGH)
}

/// What the bar says, and how full it is.
fn bar_shows(step: LoadingStep, status: &ServerStatus) -> (String, f32) {
    if let ServerStatus::Refused(reason) = status {
        return (format!("The server refused: {reason}"), 0.0);
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
            // Above everything, the debug menu too.
            GlobalZIndex(i32::MAX),
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
            screen
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(BAR_MARGIN),
                        right: px(BAR_MARGIN),
                        bottom: px(BAR_MARGIN),
                        height: px(BAR_HEIGHT),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(BAR_BACKGROUND),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        BarFill,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            top: px(0),
                            bottom: px(0),
                            width: percent(0),
                            ..default()
                        },
                        BackgroundColor(GOLD.with_alpha(0.45)),
                    ));
                    bar.spawn((
                        BarText,
                        Text::new(""),
                        TextFont::from_font_size(BAR_FONT),
                        TextColor(WRITING),
                    ));
                });
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

fn show_screen(
    step: Res<LoadingStep>,
    status: Res<ServerStatus>,
    mut screen: Single<&mut Visibility, With<LoadingScreen>>,
    mut fill: Single<&mut Node, With<BarFill>>,
    mut text: Single<(&mut Text, &mut TextColor), With<BarText>>,
) {
    if !step.is_changed() && !status.is_changed() {
        return;
    }
    screen.set_if_neq(if *step == LoadingStep::Ready {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    });
    let (words, full) = bar_shows(*step, &status);
    fill.width = percent(full * 100.0);
    text.0.0 = words;
    text.1.0 = if matches!(*status, ServerStatus::Refused(_)) {
        REFUSED
    } else {
        WRITING
    };
}

fn fit_title(
    window: Single<&Window, With<PrimaryWindow>>,
    mut title: Single<&mut TextFont, With<Title>>,
    mut last: Local<Vec2>,
) {
    let size = window.size();
    if size != *last {
        *last = size;
        title.font_size = title_font_size(size).into();
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

/// Keeps loading quick even with the window in the background, where frames are
/// otherwise a second apart.
fn keep_frame_rate(step: Res<LoadingStep>, mut settings: ResMut<WinitSettings>) {
    if step.is_changed() {
        *settings = frame_settings(*step != LoadingStep::Ready);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_lasts_until_the_screen_has_been_drawn() {
        assert_eq!(
            next_step(LoadingStep::Starting, 0, 0),
            LoadingStep::Starting
        );
        assert_eq!(
            next_step(LoadingStep::Starting, FRAMES_TO_DRAW, 0),
            LoadingStep::Connecting
        );
    }

    #[test]
    fn ready_only_once_settled_and_gone_when_the_connection_is() {
        let many = 1000;
        assert_eq!(
            next_step(LoadingStep::Connecting, many, FRAMES_TO_SETTLE),
            LoadingStep::Connecting
        );
        assert_eq!(
            next_step(LoadingStep::Connecting, many, FRAMES_TO_SETTLE + 1),
            LoadingStep::Ready
        );
        assert_eq!(
            next_step(LoadingStep::Ready, many, 0),
            LoadingStep::Connecting
        );
    }

    #[test]
    fn title_stays_in_its_share_of_any_window() {
        for window in [Vec2::new(1280.0, 720.0), Vec2::new(375.0, 812.0)] {
            let size = title_font_size(window);
            assert!(size * TITLE_EMS_WIDE <= window.x * TITLE_WIDTH_SHARE + 0.01);
            assert!(size * TITLE_EMS_HIGH <= window.y * TITLE_HEIGHT_SHARE + 0.01);
        }
    }

    #[test]
    fn a_refusal_is_said_on_the_bar() {
        let refused = ServerStatus::Refused("too old".into());
        let (words, _) = bar_shows(LoadingStep::Connecting, &refused);
        assert_eq!(words, "The server refused: too old");
    }
}
