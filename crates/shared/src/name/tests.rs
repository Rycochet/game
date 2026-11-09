use super::*;

#[test]
fn plain_names_are_kept() {
    for name in ["Brin", "van der Barbarian", "O'Brien", "Anne-Marie", "R2D2"] {
        assert_eq!(checked(name).as_deref(), Ok(name));
    }
}

#[test]
fn lookalikes_become_plain() {
    assert_eq!(fold("O\u{2019}Brien"), "O'Brien");
    assert_eq!(fold("Anne\u{2014}Marie"), "Anne-Marie");
    assert_eq!(fold("van\u{00A0}der"), "van der");
    assert_eq!(fold("Zoë Ångström"), "Zoe Angstrom");
    assert_eq!(fold("Straße Ærø Łukasz Þór"), "Strasse AEro Lukasz Thor");
    assert_eq!(fold("ＡＢＣ"), "ABC");
}

#[test]
fn what_cant_be_in_a_name_goes() {
    assert_eq!(fold("Brin!☺ the 3rd."), "Brin the 3rd");
    assert_eq!(fold("Брин"), "");
}

#[test]
fn separators_only_come_between() {
    assert_eq!(fold("  -Brin"), "Brin");
    assert_eq!(fold("van  der--Bar '' x"), "van der-Bar x");
    // One may end it while typing, but not once kept.
    assert_eq!(fold("van "), "van ");
    assert_eq!(checked("van ").as_deref(), Ok("van"));
}

#[test]
fn names_are_one_to_twenty_four_long() {
    assert!(checked("").is_err());
    assert!(checked("!!!").is_err());
    assert!(checked(&"a".repeat(MAX_NAME)).is_ok());
    assert!(checked(&"a".repeat(MAX_NAME + 1)).is_err());
}

#[test]
fn capitals_dont_make_another_name() {
    assert_eq!(key("Brin"), key("BRIN"));
    assert_ne!(key("Brin"), key("Brin 2"));
}

#[test]
fn numbered_names_fit() {
    assert_eq!(numbered("Brin", 2), "Brin 2");
    let long = numbered(&"a".repeat(MAX_NAME), 12);
    assert_eq!(long.chars().count(), MAX_NAME);
    assert!(long.ends_with(" 12"));
    assert_eq!(
        numbered("Brin of the very long hill", 3),
        "Brin of the very long 3"
    );
}
