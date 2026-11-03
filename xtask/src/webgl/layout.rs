//! Uniform sizes: every `#[derive(ShaderType)]` struct in the client, laid out
//! as WGSL lays out a uniform, must come to a multiple of 16 bytes.

use std::collections::BTreeMap;

/// A struct read from the source: where it is, and its fields' types.
pub(super) struct Found {
    pub(super) place: String,
    pub(super) fields: Vec<String>,
}

/// Each size problem, as `path:line: what`.
pub(super) fn problems(files: &[(String, String)]) -> Vec<String> {
    let mut structs = BTreeMap::new();
    for (path, text) in files {
        structs.extend(shader_types(path, text));
    }
    let mut problems = Vec::new();
    for (name, found) in &structs {
        match size_of(name, &structs) {
            Ok((size, _)) if size % 16 != 0 => problems.push(format!(
                "{}: `{name}` is {size} bytes as a uniform; WebGL 2 needs a multiple of 16: add unused `f32`s (and the same in its shader)",
                found.place
            )),
            Ok(_) => {}
            Err(why) => problems.push(format!("{}: `{name}`: {why}", found.place)),
        }
    }
    problems
}

/// The `ShaderType` structs in one file, by name.
pub(super) fn shader_types(path: &str, text: &str) -> BTreeMap<String, Found> {
    let mut found = BTreeMap::new();
    let (mut in_derive, mut wanted) = (false, false);
    let mut lines = text.lines().enumerate();
    while let Some((index, line)) = lines.next() {
        let line = line.trim();
        if line.starts_with("#[derive(") {
            in_derive = true;
        }
        if in_derive {
            wanted |= line.contains("ShaderType");
            in_derive = !line.contains(")]");
            continue;
        }
        let Some(rest) = line
            .split("struct ")
            .nth(1)
            .filter(|_| !line.starts_with("//"))
        else {
            continue;
        };
        if !std::mem::take(&mut wanted) {
            continue;
        }
        let name: String = rest
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        let mut fields = Vec::new();
        if rest.trim_end().ends_with('{') {
            for (_, field) in lines.by_ref() {
                let field = field.trim();
                if field.starts_with('}') {
                    break;
                }
                if let Some((_, kind)) = field
                    .split_once(':')
                    .filter(|_| !field.starts_with("//") && !field.starts_with('#'))
                {
                    fields.push(kind.trim().trim_end_matches(',').to_string());
                }
            }
        }
        let place = format!("{path}:{}", index + 1);
        found.insert(name, Found { place, fields });
    }
    found
}

/// A type's size and alignment as a uniform's member, in bytes.
fn size_of(name: &str, structs: &BTreeMap<String, Found>) -> Result<(u32, u32), String> {
    let short = name.rsplit("::").next().unwrap_or(name);
    if let Some(known) = basic(short) {
        return Ok(known);
    }
    let Some(found) = structs.get(short) else {
        return Err(format!(
            "unknown field type `{name}`: teach xtask/src/webgl/layout.rs its size"
        ));
    };
    if found.fields.is_empty() {
        return Err("no fields read: write it as `struct Name {` with one field per line".into());
    }
    // A struct rounds up to its largest member's alignment. Inside another
    // struct in a uniform it starts on 16 bytes, and so does what follows it.
    let (mut end, mut align) = (0_u32, 4_u32);
    for field in &found.fields {
        let (size, mut field_align) = size_of(field, structs)?;
        let nested = basic(field.rsplit("::").next().unwrap_or(field)).is_none();
        if nested {
            field_align = field_align.next_multiple_of(16);
        }
        end = end.next_multiple_of(field_align) + size;
        if nested {
            end = end.next_multiple_of(16);
        }
        align = align.max(field_align);
    }
    Ok((end.next_multiple_of(align), align))
}

/// Size and alignment of the plain types WGSL shares with Bevy's maths.
fn basic(name: &str) -> Option<(u32, u32)> {
    Some(match name {
        "f32" | "u32" | "i32" => (4, 4),
        "Vec2" | "UVec2" | "IVec2" => (8, 8),
        "Vec3" | "UVec3" | "IVec3" => (12, 16),
        "Vec4" | "UVec4" | "IVec4" | "LinearRgba" => (16, 16),
        "Mat2" => (16, 8),
        "Mat3" => (48, 16),
        "Mat4" => (64, 16),
        _ => return None,
    })
}
