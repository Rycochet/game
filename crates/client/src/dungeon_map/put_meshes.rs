//! Putting newly built meshes in place of the old.

use bevy::prelude::*;

/// Puts each built mesh (by its part's index) in place of the one drawn,
/// showing it only if it has faces and `shown`. An empty mesh is left as it
/// was, hidden: Bevy warns about meshes with no vertices.
pub(super) fn put_meshes<'a>(
    meshes: &mut Assets<Mesh>,
    built: Vec<Mesh>,
    has: &[bool],
    shown: bool,
    parts: impl Iterator<Item = (usize, &'a Mesh3d, Mut<'a, Visibility>)>,
) {
    let mut built: Vec<Option<Mesh>> = built.into_iter().map(Some).collect();
    for (i, handle, mut visibility) in parts {
        let visible = has[i] && shown;
        visibility.set_if_neq(if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if !has[i] {
            continue;
        }
        if let (Some(mut stored), Some(mesh)) = (meshes.get_mut(&handle.0), built[i].take()) {
            *stored = mesh;
        }
    }
}
