use super::files::files;
use super::*;
use crate::gender::Gender;

/// A story fact as PR 3 will add them, to show any enum can be a selector.
#[derive(Clone, Copy)]
enum Direction {
    West,
    NorthEast,
}

impl Selector for Direction {
    fn fact(&self) -> &'static str {
        "direction"
    }

    fn key(&self) -> &'static str {
        match self {
            Direction::West => "west",
            Direction::NorthEast => "north-east",
        }
    }
}

/// A language with the built-in files plus one written for a test.
fn with(extra: &'static str) -> Lang {
    let built_in = files(Language::English).iter().map(|f| (f.name, f.text));
    Lang::build(Language::English, built_in.chain([("test.ftl", extra)])).unwrap()
}

#[test]
fn every_built_in_file_parses_without_repeats() {
    for language in Language::ALL {
        let sources = files(language).iter().map(|f| (f.name, f.text));
        if let Err(problems) = Lang::build(language, sources) {
            panic!("{}: {problems:?}", language.name());
        }
    }
}

#[test]
fn every_message_renders_when_given_every_value_it_uses() {
    for language in Language::ALL {
        let lang = Lang::new(language);
        assert!(!lang.ids().is_empty());
        for id in lang.ids() {
            let attributes = lang.attributes(id).into_iter().map(|a| format!("{id}.{a}"));
            let ids = lang
                .has(id)
                .then(|| id.clone())
                .into_iter()
                .chain(attributes);
            for id in ids {
                let mut args = Args::new();
                for name in lang.variables(&id) {
                    args = args.text(name, "x");
                }
                if let Err(error) = lang.render(&id, &args) {
                    panic!("{}: {error}", language.name());
                }
            }
        }
    }
}

#[test]
fn every_gender_has_a_phrase_and_pronouns() {
    let lang = Lang::new(Language::English);
    let pronouns: Vec<String> = Gender::ALL
        .iter()
        .map(|g| {
            lang.render("pronoun-subject", &Args::new().choice("gender", g))
                .unwrap()
        })
        .collect();
    assert_eq!(pronouns, ["he", "she", "they"]);
    assert_eq!(lang.phrase(&Gender::Female).unwrap(), "female");
}

#[test]
fn values_and_selectors_fill_a_message() {
    let lang = with(
        r#"lost = { $giver-name } lost { POSSESSIVE($owner-name) } ring { $direction ->
        [west] west
        [north-east] north-east
       *[other] away
    } of here, { $count ->
        [one] one wolf
       *[other] { $count } wolves
    }{ $truth ->
        [lie] {" "}(or so { pronoun-subject } said)
       *[true] {""}
    }"#,
    );
    let args = Args::new()
        .name("giver-name", "Mara")
        .name("owner-name", "Charles")
        .choice("direction", &Direction::NorthEast)
        .count("count", 3)
        .choice("gender", &Gender::Female)
        .text("truth", "lie");
    assert_eq!(
        lang.render("lost", &args).unwrap(),
        "Mara lost Charles' ring north-east of here, 3 wolves (or so she said)"
    );
    let one = Args::new()
        .name("giver-name", "Mara")
        .name("owner-name", "Ida")
        .choice("direction", &Direction::West)
        .count("count", 1)
        .choice("gender", &Gender::Male)
        .text("truth", "true");
    assert_eq!(
        lang.render("lost", &one).unwrap(),
        "Mara lost Ida's ring west of here, one wolf"
    );
}

#[test]
fn variables_include_those_of_referred_messages() {
    let lang = with("said = { $name } said { pronoun-object }");
    let names: Vec<String> = lang.variables("said").into_iter().collect();
    assert_eq!(names, ["gender", "name"]);
    assert_eq!(
        lang.missing("said", &Args::new().name("name", "Ida")),
        ["gender"]
    );
}

#[test]
fn a_missing_value_is_an_error_showing_its_name() {
    let lang = Lang::new(Language::English);
    let error = lang.render("pronoun-subject", &Args::new()).unwrap_err();
    assert_eq!(error.id, "pronoun-subject");
    assert!(
        error.text.contains("they"),
        "falls to the default: {}",
        error.text
    );
    assert!(!error.problems.is_empty());
    assert!(lang.render("no-such-message", &Args::new()).is_err());
}

#[test]
fn rule_functions_work_on_values_terms_and_messages() {
    let lang = with(
        "-ring = old ring\n\
         item = axe\n\
         found = { CAP(A(-ring)) } and { A(item) }, { A($thing) }, { POSSESSIVE($name) }",
    );
    let args = Args::new().text("thing", "hour").name("name", "Hess");
    assert_eq!(
        lang.render("found", &args).unwrap(),
        "An old ring and an axe, an hour, Hess'"
    );
}

#[test]
fn attributes_hold_wordings() {
    let lang = with("beat = { $n }\n    .one = first { $n }\n    .two = second");
    assert_eq!(lang.attributes("beat"), ["one", "two"]);
    assert_eq!(
        lang.render("beat.one", &Args::new().count("n", 2)).unwrap(),
        "first 2"
    );
    assert!(lang.has("beat.two"));
    assert!(!lang.has("beat.three"));
}

#[test]
fn no_isolation_marks_round_values() {
    let lang = with("hi = Hello, { $name }!");
    assert_eq!(
        lang.render("hi", &Args::new().name("name", "Ida")).unwrap(),
        "Hello, Ida!"
    );
}

#[test]
fn broken_files_and_repeats_are_refused() {
    assert!(Lang::build(Language::English, [("a.ftl", "broken = {")]).is_err());
    let twice = [("a.ftl", "x = 1"), ("b.ftl", "x = 2")];
    assert!(Lang::build(Language::English, twice).is_err());
}

#[test]
fn languages_from_locales() {
    assert_eq!(Language::from_locale("en-GB"), Language::English);
    assert_eq!(Language::from_locale("EN_us"), Language::English);
    assert_eq!(Language::from_locale("fr-FR"), Language::English);
    assert_eq!(Language::from_locale(""), Language::English);
}

#[test]
fn lang_can_be_shared_between_threads() {
    fn shareable<T: Send + Sync>() {}
    shareable::<Lang>();
}
