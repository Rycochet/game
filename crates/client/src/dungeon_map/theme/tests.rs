use super::*;

/// Every picture a theme names is in the assets folder.
#[test]
fn pictures_exist() {
    let assets = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/");
    let kinds = PoiKind::ALL.into_iter().map(Some).chain([None]);
    for theme in kinds.map(Theme::of) {
        for surface in Surface::ALL {
            let path = format!("{assets}{}", theme.picture(surface));
            assert!(std::path::Path::new(&path).exists(), "missing {path}");
        }
    }
}

/// Walls are drawn natural exactly where the shared rules block them as
/// natural (`PoiKind::natural_walls`), so they block where they are drawn.
#[test]
fn natural_walls_match_the_shared_rule() {
    for kind in PoiKind::ALL {
        let natural = Theme::of(Some(kind)).shape == Shape::Natural;
        assert_eq!(natural, kind.natural_walls(), "{kind:?}");
    }
    assert_eq!(
        Theme::of_level(Some(LevelKind::Wilderness)).shape,
        Shape::Built
    );
}
