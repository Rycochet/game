use super::super::others::near_way_round;
use super::super::projection::Projection;
use super::super::spawn::new_picture;
use super::*;
use crate::common::{Attitude, Token};

const SIZE: u32 = 100;

/// A north-up map, a cell to 10 pixels, centred on (50, 50).
fn under(inside: Option<((f32, f32), f32)>) -> Under {
    Under {
        projection: Projection {
            centre: (50.0, 50.0),
            middle: (50.0, 50.0),
            units_per_pixel: 0.1,
            right: Projection::NORTH_UP.0,
            down: Projection::NORTH_UP.1,
        },
        size: (SIZE, SIZE),
        full_map: inside.is_none(),
        inside,
        mark: 1.0,
        other_floor: None,
        wrap: None,
    }
}

fn painted(from: &MarksFrom) -> Vec<u8> {
    let mut pixels = vec![0; (SIZE * SIZE * 4) as usize];
    let mut canvas = Canvas {
        pixels: &mut pixels,
        width: SIZE,
        height: SIZE,
    };
    paint(&mut canvas, from);
    pixels
}

fn pixel(pixels: &[u8], (x, y): (u32, u32)) -> [u8; 4] {
    let at = ((y * SIZE + x) * 4) as usize;
    [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]]
}

#[test]
fn others_are_dots_in_their_attitudes_colour_with_a_black_edge() {
    let others = vec![
        ((52, 50), Attitude::Unfriendly),
        ((48, 50), Attitude::Friendly),
    ];
    let pixels = painted(&marks_from(under(None), (50.0, 50.0), others));
    assert_eq!(pixel(&pixels, (70, 50)), [0xee, 0x33, 0x33, 255]);
    assert_eq!(pixel(&pixels, (30, 50)), [0x33, 0xcc, 0x33, 255]);
    // Its edge: the dot is 10 pixels across, its outline 1.
    let [red, green, blue, alpha] = pixel(&pixels, (70, 45));
    assert_eq!([red, green, blue], [0, 0, 0]);
    assert!(alpha > 200);
    // The player, white, in the middle; nothing between them.
    assert_eq!(pixel(&pixels, (50, 50)), [255, 255, 255, 255]);
    assert_eq!(pixel(&pixels, (60, 50))[3], 0);
}

#[test]
fn the_player_is_drawn_over_anyone_in_the_same_place() {
    let others = vec![((50, 50), Attitude::Unfriendly)];
    let pixels = painted(&marks_from(under(None), (50.0, 50.0), others));
    assert_eq!(pixel(&pixels, (50, 50)), [255, 255, 255, 255]);
}

#[test]
fn on_the_minimap_dots_are_cut_off_at_the_border() {
    // The circle inside the border reaches 20 pixels from the middle.
    let others = vec![((52, 50), Attitude::Neutral)];
    let pixels = painted(&marks_from(
        under(Some(((50.0, 50.0), 20.0))),
        (50.0, 50.0),
        others,
    ));
    assert_eq!(pixel(&pixels, (67, 50)), [0x33, 0x88, 0xff, 255]);
    assert_eq!(pixel(&pixels, (73, 50))[3], 0);
}

#[test]
fn on_the_world_map_others_are_placed_the_near_way_round() {
    assert_eq!(near_way_round(2, 998.0, Some(1000)), 1002);
    assert_eq!(near_way_round(998, 2.0, Some(1000)), -2);
    assert_eq!(near_way_round(500, 400.0, Some(1000)), 500);
    assert_eq!(near_way_round(2, 998.0, None), 2);
}

/// An app with only the marks layer, over a map drawn in grey.
fn marks_app() -> (App, MapPictures, Entity) {
    let mut app = App::new();
    app.init_resource::<Assets<Image>>()
        .init_resource::<CurrentDungeon>()
        .init_resource::<OtherLevels>()
        .init_resource::<crate::quests::TrackedDots>()
        .insert_resource(MarksLayer(Some(under(Some(((50.0, 50.0), 50.0))))))
        .add_systems(Update, draw_marks);
    let mut images = app.world_mut().resource_mut::<Assets<Image>>();
    let mut base = new_picture();
    base.data = Some(vec![128; 4]);
    let pictures = MapPictures {
        minimap: images.add(base),
        full: images.add(new_picture()),
        minimap_marks: images.add(new_picture()),
        full_marks: images.add(new_picture()),
    };
    app.insert_resource(MapPictures {
        minimap: pictures.minimap.clone(),
        full: pictures.full.clone(),
        minimap_marks: pictures.minimap_marks.clone(),
        full_marks: pictures.full_marks.clone(),
    });
    app.world_mut()
        .spawn((Player, SmoothPosition { x: 50.0, y: 50.0 }));
    let mob = app
        .world_mut()
        .spawn((
            Other {
                id: shared::id::Id::for_test(1),
                kind: shared::creature::CreatureKind::Kobold,
                feeling: shared::sheet::Feeling::Unfriendly,
            },
            MapPosition { x: 52, y: 50 },
            Token {
                attitude: Attitude::Unfriendly,
                ..default()
            },
        ))
        .id();
    (app, pictures, mob)
}

fn layer_pixel(app: &App, picture: &Handle<Image>, at: (u32, u32)) -> [u8; 4] {
    let image = app
        .world()
        .resource::<Assets<Image>>()
        .get(picture)
        .unwrap();
    pixel(image.data.as_ref().unwrap(), at)
}

#[test]
fn someone_moving_redraws_only_the_marks_layer() {
    let (mut app, pictures, mob) = marks_app();
    app.update();
    assert_eq!(
        layer_pixel(&app, &pictures.minimap_marks, (70, 50))[0],
        0xee
    );
    app.world_mut()
        .entity_mut(mob)
        .insert(MapPosition { x: 50, y: 53 });
    app.update();
    assert_eq!(layer_pixel(&app, &pictures.minimap_marks, (70, 50))[3], 0);
    assert_eq!(
        layer_pixel(&app, &pictures.minimap_marks, (50, 80))[0],
        0xee
    );
    // The map under it is left as it was.
    let base = app.world().resource::<Assets<Image>>();
    let base = base.get(&pictures.minimap).unwrap();
    assert_eq!(base.data.as_deref(), Some(&[128u8; 4][..]));
}

#[test]
fn on_another_floor_only_the_player_is_marked() {
    let (mut app, pictures, _) = marks_app();
    let mut layer = app.world_mut().resource_mut::<MarksLayer>();
    if let Some(under) = &mut layer.0 {
        under.other_floor = Some(true);
    }
    app.update();
    assert_eq!(layer_pixel(&app, &pictures.minimap_marks, (70, 50))[3], 0);
}
