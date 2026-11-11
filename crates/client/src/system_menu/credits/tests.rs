use super::*;

/// The website's credits page.
const PAGE: &str = include_str!("../../../../../docs/credits.md");

/// Everyone and everything the page credits by name (its bold labels) is in
/// the game's credits too.
#[test]
fn credits_everything_the_website_does() {
    let ours: String = CREDITS
        .iter()
        .map(|(what, who)| format!("{what} {who}\n"))
        .collect();
    let labels: Vec<&str> = PAGE
        .lines()
        .filter_map(|line| line.strip_prefix("- **")?.split_once(":**"))
        .map(|(label, _)| label)
        .collect();
    assert!(!labels.is_empty());
    for label in labels {
        assert!(
            ours.contains(label),
            "the game's credits leave out {label:?}"
        );
    }
}

#[test]
fn the_layer_is_good_markup() {
    CREDITS_LAYER.assert_good();
}
