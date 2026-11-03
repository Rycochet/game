use super::*;

#[test]
fn a_picture_marks_the_player_and_explains_its_letters() {
    let level = Level::test_room();
    let (min_x, min_y, ..) = level.bounds();
    let (rows, legend) = picture(&level, (min_x + 1, min_y + 1), 1);
    assert_eq!(rows, ["###", "#@.", "#.."]);
    assert_eq!(legend, json!({ "@": "player", "#": "Wall", ".": "Floor" }));
    // Outside the level is blank.
    let (rows, _) = picture(&level, (min_x, min_y), 1);
    assert_eq!(rows, ["   ", " @#", " #."]);
}
