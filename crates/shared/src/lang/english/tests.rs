use super::*;

#[test]
fn a_or_an_by_how_the_word_is_said() {
    let e = English;
    assert_eq!(e.indefinite("sword"), "a sword");
    assert_eq!(e.indefinite("axe"), "an axe");
    assert_eq!(e.indefinite("old ring"), "an old ring");
    assert_eq!(e.indefinite("Iron Key"), "an Iron Key");
    assert_eq!(e.indefinite("unicorn"), "a unicorn");
    assert_eq!(e.indefinite("useful tool"), "a useful tool");
    assert_eq!(e.indefinite("one-eyed wolf"), "a one-eyed wolf");
    assert_eq!(e.indefinite("ewer"), "a ewer");
    assert_eq!(e.indefinite("hour"), "an hour");
    assert_eq!(e.indefinite("heirloom"), "an heirloom");
    assert_eq!(e.indefinite("herb"), "a herb");
    assert_eq!(e.indefinite("urn"), "an urn");
    assert_eq!(e.indefinite(""), "");
}

#[test]
fn possessives_after_an_s_take_only_the_apostrophe() {
    let e = English;
    assert_eq!(e.possessive("Mara"), "Mara's");
    assert_eq!(e.possessive("Charles"), "Charles'");
    assert_eq!(e.possessive("ROSS"), "ROSS'");
}

#[test]
fn lists_join_with_commas_and_a_last_and() {
    let e = English;
    assert_eq!(e.list(&[]), "");
    assert_eq!(e.list(&["wolves"]), "wolves");
    assert_eq!(e.list(&["wolves", "bats"]), "wolves and bats");
    assert_eq!(
        e.list(&["wolves", "bats", "a bear"]),
        "wolves, bats and a bear"
    );
}

#[test]
fn capitals_go_on_the_first_letter() {
    let e = English;
    assert_eq!(e.capitalise("a ring"), "A ring");
    assert_eq!(e.capitalise("\"go west\""), "\"Go west\"");
    assert_eq!(e.capitalise("élan"), "Élan");
    assert_eq!(e.capitalise("123"), "123");
    assert_eq!(e.capitalise(""), "");
}
