use super::*;

fn find(path: &str) -> SkillId {
    SkillId::find(path).unwrap()
}

#[test]
fn nothing_known_places_nothing() {
    assert!(layout(&Skills::default()).is_empty());
}

#[test]
fn known_top_levels_share_the_circle_in_order_by_the_root_of_what_ends_under_them() {
    let mut skills = Skills::default();
    skills.practise(find("athletics/climbing"), 100);
    skills.practise(find("athletics/running"), 100);
    skills.practise(find("lore/history"), 100);
    let placed = layout(&skills);
    let at = |path: &str| placed.iter().find(|one| one.id == find(path)).unwrap().at;
    // Athletics has two ends and Lore one: they share the circle by the square
    // roots, from the top.
    let athletics = 360.0 * 2f32.sqrt() / (2f32.sqrt() + 1.0);
    for (path, degrees) in [
        ("athletics", athletics / 2.0),
        ("lore", (athletics + 360.0) / 2.0),
    ] {
        let angle = f32::to_radians(degrees);
        let expected = Vec2::new(angle.sin(), -angle.cos()) * RINGS[1];
        assert!(at(path).distance(expected) < 0.01, "{path}");
    }
}

#[test]
fn only_known_skills_are_placed_parents_first_on_their_rings() {
    let mut skills = Skills::default();
    skills.practise(find("combat/melee/longsword/longsword"), 1000);
    skills.practise(find("combat/melee/defence/buckler"), 1000);
    let placed = layout(&skills);
    let known = skills.known().count();
    assert_eq!(placed.len(), known);
    for (index, one) in placed.iter().enumerate() {
        let ring = RINGS[usize::from(one.id.depth())];
        assert!((one.at.length() - ring).abs() < 0.01, "{}", one.id.path());
        match one.id.parent() {
            Some(parent) => {
                let parent_at = placed[..index].iter().find(|p| p.id == parent).unwrap();
                assert_eq!(one.from, parent_at.at);
            }
            None => assert_eq!(one.from, Vec2::ZERO),
        }
    }
    let reach = placed.iter().map(|one| one.at.length()).fold(0.0, f32::max);
    assert!((reach - RINGS[4]).abs() < 0.01);
}

#[test]
fn siblings_share_their_parents_slice_by_what_ends_under_them() {
    let mut skills = Skills::default();
    skills.practise(find("athletics/climbing"), 100);
    skills.practise(find("athletics/running"), 100);
    let placed = layout(&skills);
    let at = |path: &str| placed.iter().find(|one| one.id == find(path)).unwrap().at;
    // Athletics alone has the whole circle, so it sits at the bottom (180
    // degrees); its two ends share the 90 degrees its children may spread over,
    // a half each, round its line.
    for (path, degrees) in [
        ("athletics/climbing", 180.0 - 22.5),
        ("athletics/running", 180.0 + 22.5),
    ] {
        let angle = f32::to_radians(degrees);
        let expected = Vec2::new(angle.sin(), -angle.cos()) * RINGS[2];
        assert!(at(path).distance(expected) < 0.01, "{path}");
    }
}

/// Every node is further out than its parent, along its parent's own line.
fn assert_outwards(placed: &[Placed]) {
    for one in placed {
        if one.from == Vec2::ZERO {
            continue;
        }
        let along = (one.at - one.from).dot(one.from.normalize());
        assert!(along > 0.0, "{} points back ({along})", one.id.path());
    }
}

#[test]
fn branches_always_point_outwards() {
    // One branch alone, as a new character has (the owner's screenshot).
    let mut alone = Skills::default();
    for path in ["athletics/climbing", "athletics/swimming"] {
        alone.practise(find(path), 100);
    }
    assert_outwards(&layout(&alone));
    // One deep branch with many ends.
    let mut melee = Skills::default();
    for &child in find("combat/melee/longsword").children() {
        melee.practise(child, 100);
    }
    melee.practise(find("combat/melee/defence/buckler"), 100);
    assert_outwards(&layout(&melee));
    // Everything known at once.
    let mut everything = Skills::default();
    for id in SkillId::all().filter(|id| id.children().is_empty()) {
        everything.practise(id, 100);
    }
    assert_outwards(&layout(&everything));
}
