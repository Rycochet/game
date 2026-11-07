use super::*;

fn rgb(colour: Color) -> [f32; 3] {
    let colour = colour.to_srgba();
    [colour.red, colour.green, colour.blue]
}

#[test]
fn one_row_takes_its_colour_and_two_rows_mix() {
    use Attribute::*;
    let body = blend(&[Strength, Agility]);
    assert_eq!(body, blend(&[Toughness]));
    let half = blend(&[Strength, Perception]);
    let (red, blue) = (blend(&[Strength]), blend(&[Perception]));
    for index in 0..3 {
        assert!((half[index] - (red[index] + blue[index]) / 2.0).abs() < 1e-6);
    }
}

#[test]
fn active_is_darker_and_passive_lighter_than_the_row() {
    let [red, green, blue] = row_rgb(Row::Body);
    let row = rgb(Color::srgb_u8(red, green, blue));
    let active = rgb(attribute_colour(&[Attribute::Strength], false));
    let passive = rgb(attribute_colour(&[Attribute::Strength], true));
    for index in 0..3 {
        assert!(active[index] < row[index] + 1e-6);
        assert!(passive[index] > row[index] - 1e-6);
    }
    // Leaning on nothing still has a colour.
    assert_ne!(attribute_colour(&[], false), Color::BLACK);
}
