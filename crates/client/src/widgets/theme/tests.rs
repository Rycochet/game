use super::*;

#[test]
fn text_sizes_go_up_in_order() {
    let sizes = [
        TextSize::Small,
        TextSize::Medium,
        TextSize::Large,
        TextSize::Title,
    ];
    assert!(
        sizes
            .windows(2)
            .all(|two| two[0].pixels() < two[1].pixels())
    );
}
