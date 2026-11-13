use super::*;

/// The website's credits page.
const PAGE: &str = include_str!("../../../../../docs/credits.md");

/// Everyone and everything the page credits by name (its bold labels) is in
/// the game's credits too.
#[test]
fn credits_everything_the_website_does() {
    let ours: String = CREDITS
        .iter()
        .flat_map(|(_, entries)| entries.iter())
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

/// The page's headings are the game's, in the same order.
#[test]
fn the_website_groups_as_the_game_does() {
    let ours: Vec<&str> = CREDITS
        .iter()
        .map(|(heading, _)| *heading)
        .filter(|heading| !heading.is_empty())
        .collect();
    let page: Vec<&str> = PAGE
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .collect();
    assert_eq!(page, ours);
}

#[test]
fn the_layer_is_good_markup() {
    CREDITS_LAYER.assert_good();
}
