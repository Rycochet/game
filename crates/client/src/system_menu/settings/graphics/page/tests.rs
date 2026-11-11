use super::*;
use crate::common::{
    AntiAliasing, Background, DisplayMode, FrameRate, MaxResolution, Precipitation, Preset,
    Remembered, Shadows, ViewDistance,
};
use crate::dom::{CheckedNode, Tag};

/// The words of each `option` of the select with this name, in order.
fn options(nodes: &[CheckedNode], name: &str, found: &mut Vec<String>) {
    for node in nodes {
        let CheckedNode::Element(element) = node else {
            continue;
        };
        let named = element.id.as_ref().is_some_and(|id| id.parts().1 == name);
        if element.tag == Tag::Select && named {
            for option in &element.children {
                if let CheckedNode::Element(option) = option
                    && let Some(CheckedNode::Text(words)) = option.children.first()
                {
                    found.push(words.clone());
                }
            }
        }
        options(&element.children, name, found);
    }
}

fn names<T: Copy>(all: &[T], name: impl Fn(T) -> &'static str) -> Vec<String> {
    all.iter().map(|&it| name(it).to_owned()).collect()
}

#[test]
fn every_select_offers_its_settings_choices() {
    let nodes = GRAPHICS.checked().expect("good markup");
    let offered = |name: &str| {
        let mut found = Vec::new();
        options(&nodes, name, &mut found);
        found
    };
    let mut presets = names(&Preset::ALL, Preset::name);
    presets.push("Custom".into());
    assert_eq!(offered("preset"), presets);
    let modes = names(&DisplayMode::ALL, DisplayMode::name);
    assert_eq!(offered("display_mode"), modes);
    let rates = names(&FrameRate::ALL, FrameRate::name);
    assert_eq!(offered("frame_rate"), rates);
    let most = names(&MaxResolution::ALL, MaxResolution::name);
    assert_eq!(offered("max_resolution"), most);
    let smoothing = names(&AntiAliasing::ALL, AntiAliasing::name);
    assert_eq!(offered("anti_aliasing"), smoothing);
    assert_eq!(offered("shadows"), names(&Shadows::ALL, Shadows::name));
    let distances = names(&ViewDistance::ALL, ViewDistance::name);
    assert_eq!(offered("view_distance"), distances);
    let weather = names(&Precipitation::ALL, Precipitation::name);
    assert_eq!(offered("precipitation"), weather);
    let remembered = names(&Remembered::ALL, Remembered::name);
    assert_eq!(offered("remembered"), remembered);
    let background = names(&Background::ALL, Background::name);
    assert_eq!(offered("background"), background);
}

#[test]
fn every_setting_has_its_control() {
    use super::super::options::GraphicsControl;
    let text = GRAPHICS.source;
    for control in GraphicsControl::ALL {
        let id = format!("id=\"{}\"", control.ui_name());
        assert!(text.contains(&id), "{control:?}");
    }
}
