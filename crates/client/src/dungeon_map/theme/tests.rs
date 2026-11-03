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
