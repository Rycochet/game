use shared::creature::CreatureKind;
use shared::sheet::Appearance;

use super::bones::BodyPart;
use super::*;
use crate::item_art::character_look;

fn build(kind: CreatureKind) -> Option<TokenModel> {
    let look = character_look(kind, &Appearance::default(), &[])?;
    let (mut meshes, mut materials) = (Assets::default(), Assets::default());
    CreatureModels::default().model(&look, &mut meshes, &mut materials)
}

#[test]
fn every_kind_with_a_body_has_a_model_at_its_own_height() {
    for kind in CreatureKind::ALL {
        let Some(model) = build(kind) else {
            continue;
        };
        assert!(!model.0.parts.is_empty(), "{kind:?}");
        if kind.def().playable && shared::body::Frame::of(kind).is_some() {
            let usual = shared::body::Frame::usual_height(kind) as f32;
            let height = model.0.height;
            // Hair, horns and crests stand over the frame's height.
            assert!(
                height > usual * 0.9 && height < usual * 1.3,
                "{kind:?}: {height} for {usual}"
            );
        }
    }
}

#[test]
fn a_look_is_built_once() {
    let look = character_look(CreatureKind::Human, &Appearance::default(), &[]).unwrap();
    let (mut meshes, mut materials) = (Assets::default(), Assets::default());
    let mut models = CreatureModels::default();
    let first = models.model(&look, &mut meshes, &mut materials).unwrap();
    let again = models.model(&look, &mut meshes, &mut materials).unwrap();
    assert_eq!(first, again);
    assert_eq!(models.len(), 1);
}

#[test]
fn points_move_with_the_bone_they_are_on() {
    let layout = shared::body::Layout::at_size(CreatureKind::Human).unwrap();
    let bones = bones::Bones::of(&layout);
    let chin = layout.chin();
    let head = Vec3::new(chin[0] as f32, chin[1] as f32 + 100.0, chin[2] as f32);
    assert_eq!(bones.part_at(head), BodyPart::HEAD);
    let hips = layout.spine.at(0);
    let belly = Vec3::new(
        hips[0] as f32,
        hips[1] as f32 + 150.0,
        hips[2] as f32 + 60.0,
    );
    assert_eq!(bones.part_at(belly), BodyPart::TRUNK);
}
