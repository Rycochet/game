//! A MakeHuman target: how far each point of the base mesh moves to make
//! another shape (a cow, a dog's head).
//!
//! The file is plain text, a line each: `<point> <dx> <dy> <dz>`, the point's
//! number counting from 0. Lines starting `#` are comments.

use super::mesh::Mesh;

/// Moves the mesh's points as the target's text says.
pub fn apply(mesh: &mut Mesh, text: &str) -> Result<(), String> {
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let wrong = |what: &str| format!("line {}: {what}: {line}", number + 1);
        let mut words = line.split_whitespace();
        let at: usize = words
            .next()
            .and_then(|word| word.parse().ok())
            .ok_or_else(|| wrong("no point number"))?;
        let point = mesh
            .points
            .get_mut(at)
            .ok_or_else(|| wrong("no such point in the base mesh"))?;
        for value in point {
            let moved: f64 = words
                .next()
                .and_then(|word| word.parse().ok())
                .ok_or_else(|| wrong("not three distances"))?;
            *value += moved;
        }
    }
    Ok(())
}
