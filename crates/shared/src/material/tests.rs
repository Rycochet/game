use super::*;

#[test]
fn every_material_is_listed_alphabetically() {
    let names: Vec<_> = Material::ALL
        .iter()
        .map(|material| material.name())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

#[test]
fn the_gems_are_all_the_materials_of_that_category_cheapest_first() {
    let gems = Material::ALL
        .iter()
        .filter(|material| material.category() == Category::Gem);
    assert_eq!(gems.count(), Material::GEMS.len());
    assert!(
        Material::GEMS
            .windows(2)
            .all(|pair| pair[0].category() == Category::Gem && pair[0].tier() <= pair[1].tier())
    );
}
