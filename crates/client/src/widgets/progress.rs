//! A bar filling up as something gets done, with words over it.

use bevy::prelude::*;

/// How full a bar is, `0..=1`. Set it to change the bar.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Progress(pub f32);

/// The part of the bar `.0` that fills up.
#[derive(Component)]
pub(super) struct ProgressFill(Entity);

/// A bar `full` of the way along, in these colours, with `node` for its size and
/// where it goes. `words` is the writing over it, with a marker to find it by.
pub fn spawn_progress_bar<'a>(
    parent: &'a mut ChildSpawnerCommands,
    full: f32,
    (track, fill): (Color, Color),
    node: Node,
    words: impl Bundle,
) -> EntityCommands<'a> {
    let mut bar = parent.spawn((
        Progress(full),
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..node
        },
        BackgroundColor(track),
    ));
    let id = bar.id();
    bar.with_children(|bar| {
        bar.spawn((
            ProgressFill(id),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                bottom: px(0),
                width: width_of(full),
                ..default()
            },
            BackgroundColor(fill),
        ));
        bar.spawn(words);
    });
    bar
}

fn width_of(full: f32) -> Val {
    percent(full.clamp(0.0, 1.0) * 100.0)
}

/// Makes bars as full as they say.
pub(super) fn show_progress(bars: Query<&Progress>, mut fills: Query<(&ProgressFill, &mut Node)>) {
    for (fill, mut node) in &mut fills {
        if let Ok(progress) = bars.get(fill.0) {
            let width = width_of(progress.0);
            if node.width != width {
                node.width = width;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bar_never_goes_past_its_ends() {
        assert_eq!(width_of(0.5), percent(50));
        assert_eq!(width_of(-1.0), percent(0));
        assert_eq!(width_of(2.0), percent(100));
    }
}
