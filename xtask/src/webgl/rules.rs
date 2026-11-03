//! The WebGL 2 rules read straight from the text: render target formats, and
//! what shaders may use.

/// Formats WebGL 2 can't draw to as a camera's target: for these Bevy gives
/// the camera's own textures an extra sRGB view, which needs `VIEW_FORMATS`.
const PLAIN_FORMATS: &[&str] = &["Rgba8Unorm", "Bgra8Unorm"];

/// What WebGL 2 has no way to run, with why, as written in a shader.
const SHADER_FEATURES: &[(&str, &str)] = &[
    ("var<storage", "storage buffers"),
    ("texture_storage", "storage textures"),
    ("@compute", "compute shaders"),
];

/// Render targets in one Rust file that WebGL 2 can't use, as `path:line: what`.
pub(super) fn target_problems(path: &str, text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    // How deep in a `new_target_texture(...)` call's brackets, over lines.
    let mut depth = 0;
    for (index, line) in text.lines().enumerate() {
        let place = format!("{path}:{}", index + 1);
        if line.contains("RENDER_ATTACHMENT") {
            problems.push(format!(
                "{place}: make render targets with `Image::new_target_texture`, so their format is checked"
            ));
        }
        let call = line.find("new_target_texture(");
        if call.is_none() && depth == 0 {
            continue;
        }
        if plain_format(line) {
            problems.push(format!(
                "{place}: a render target in a plain 8-bit format quits the game on WebGL 2: use its `Srgb` format"
            ));
        }
        let rest = call.map_or(line, |at| &line[at..]);
        for c in rest.chars() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
        }
    }
    problems
}

/// Whether a line names `Rgba8Unorm` or `Bgra8Unorm` without `Srgb` after it.
pub(super) fn plain_format(line: &str) -> bool {
    PLAIN_FORMATS.iter().any(|format| {
        line.match_indices(format)
            .any(|(at, _)| !line[at + format.len()..].starts_with("Srgb"))
    })
}

/// What one shader uses that WebGL 2 lacks, as `path:line: what`.
pub(super) fn shader_problems(path: &str, text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let code = line.split("//").next().unwrap_or_default();
        for (feature, what) in SHADER_FEATURES {
            if code.contains(feature) {
                problems.push(format!(
                    "{path}:{}: WebGL 2 has no {what} (`{feature}`)",
                    index + 1
                ));
            }
        }
    }
    problems
}
