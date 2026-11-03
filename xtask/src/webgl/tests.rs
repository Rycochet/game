use super::layout::{problems, shader_types};
use super::rules::{plain_format, shader_problems, target_problems};

fn file(text: &str) -> Vec<(String, String)> {
    vec![("a.rs".to_string(), text.to_string())]
}

#[test]
fn vec2s_and_an_f32_come_to_24_bytes() {
    let text = "#[derive(ShaderType, Clone)]\nstruct Mask {\n    /// Corner.\n    pub(super) origin: Vec2,\n    per_unit: Vec2,\n    side: f32,\n}\n";
    let found = problems(&file(text));
    assert_eq!(found.len(), 1);
    assert!(found[0].starts_with("a.rs:2:"), "{found:?}");
    assert!(found[0].contains("24 bytes"), "{found:?}");
}

#[test]
fn padding_to_a_multiple_of_16_passes() {
    let text = "#[derive(ShaderType)]\nstruct Mask {\n    origin: Vec2,\n    per_unit: Vec2,\n    side: f32,\n    _a: f32,\n    _b: f32,\n    _c: f32,\n}\n";
    assert!(problems(&file(text)).is_empty());
}

#[test]
fn a_vec4_rounds_the_struct_up_to_16() {
    let text = "#[derive(ShaderType)]\nstruct Edge {\n    colour: Vec4,\n    shift: f32,\n}\n";
    assert!(problems(&file(text)).is_empty());
}

#[test]
fn a_lone_f32_is_too_small() {
    let text = "#[derive(\n    ShaderType,\n    Debug,\n)]\nstruct Edge {\n    radius: f32,\n}\n";
    assert_eq!(problems(&file(text)).len(), 1);
}

#[test]
fn nested_structs_start_and_end_on_16() {
    let text = "#[derive(ShaderType)]\nstruct Inner {\n    a: f32,\n}\n#[derive(ShaderType)]\nstruct Outer {\n    inner: Inner,\n    b: f32,\n    c: Vec3,\n}\n";
    // Inner itself is 4 bytes; Outer is 16 + 4, padded, + 12 at 32 = 48.
    let found = problems(&file(text));
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("`Inner`"), "{found:?}");
}

#[test]
fn unknown_types_are_refused() {
    let text = "#[derive(ShaderType)]\nstruct A {\n    when: Duration,\n}\n";
    let found = problems(&file(text));
    assert!(
        found[0].contains("unknown field type `Duration`"),
        "{found:?}"
    );
}

#[test]
fn other_structs_are_left_alone() {
    let text = "#[derive(Asset, AsBindGroup)]\nstruct Material {\n    params: f32,\n}\n";
    assert!(shader_types("a.rs", text).is_empty());
}

#[test]
fn plain_target_formats_are_found() {
    assert!(plain_format("TextureFormat::Rgba8Unorm,"));
    assert!(plain_format("TextureFormat::Bgra8Unorm)"));
    assert!(!plain_format("TextureFormat::Rgba8UnormSrgb,"));
}

#[test]
fn render_targets_are_checked_across_lines() {
    let text = "let a = Image::new_target_texture(\n    8,\n    8,\n    TextureFormat::Rgba8Unorm,\n    None,\n);\nlet b = TextureFormat::Rgba8Unorm;\n";
    let found = target_problems("a.rs", text);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].starts_with("a.rs:4:"), "{found:?}");
}

#[test]
fn hand_made_render_targets_are_refused() {
    let text = "usage: TextureUsages::RENDER_ATTACHMENT,\n";
    assert_eq!(target_problems("a.rs", text).len(), 1);
}

#[test]
fn shaders_without_webgl_features_are_refused() {
    let text = "// no var<storage here\n@group(0) @binding(0) var<storage, read> a: array<f32>;\n@compute @workgroup_size(8)\n";
    let found = shader_problems("a.wesl", text);
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(found[0].starts_with("a.wesl:2:"), "{found:?}");
}
