//! The glow of an item that is magical, blessed or cursed
//! (`assets/shaders/item_aura.wesl`), never past the item's frame: round the outline
//! of a picture, behind it, or in from the edges of plain cells, over them.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;
use shared::item::{Favour, Item, Shape};

use super::Floating;
use super::item::{CELL, PICTURE_PIXELS_PER_CELL};
use crate::item_art::{ItemPictures, Look};

/// How far a glow reaches from an outline, in logical pixels.
const SPREAD: f32 = 4.0;
/// How many pixels a shape's picture has for each cell.
const MASK_PIXELS_PER_CELL: usize = 8;
const MAGICAL: Color = Color::srgb(0.2, 0.85, 1.0);
const BLESSED: Color = Color::srgb(1.0, 1.0, 0.9);
const CURSED: Color = Color::srgb(0.75, 0.0, 0.05);

/// What makes something glow, as far as this client knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Aura {
    magical: bool,
    favour: Option<Favour>,
}

impl Aura {
    /// What this item glows with, if anything.
    pub fn of(item: &Item) -> Option<Self> {
        let magical = item.is_magical();
        (magical || item.favour.is_some()).then_some(Self {
            magical,
            favour: item.favour,
        })
    }

    /// The colours it fades between: the same one twice if it has only one.
    fn colours(self) -> (Color, Color) {
        let favour = self.favour.map(|favour| match favour {
            Favour::Blessed => BLESSED,
            Favour::Cursed => CURSED,
        });
        match (self.magical, favour) {
            (true, Some(favour)) => (MAGICAL, favour),
            (false, Some(favour)) => (favour, favour),
            _ => (MAGICAL, MAGICAL),
        }
    }
}

/// What a glow takes its outline from.
#[derive(Clone, Debug)]
pub enum AuraMask {
    /// The item's picture.
    Picture(Look, u32),
    /// The cells of something with no picture.
    Cells(Shape),
}

/// On a node drawn as a glow.
#[derive(Component, Clone, Debug)]
pub struct AuraGlow {
    aura: Aura,
    mask: AuraMask,
}

/// A glow filling this node, which should be the inside of the item's frame. For a
/// picture it goes before the picture, and for plain cells after them. It is not what
/// the pointer is on.
pub fn aura_glow(aura: Aura, mask: AuraMask, node: Node) -> impl Bundle {
    (
        AuraGlow { aura, mask },
        Floating,
        Node {
            position_type: PositionType::Absolute,
            ..node
        },
    )
}

/// Order matches `struct Aura` in the shader.
#[derive(ShaderType, Clone, Copy, Debug, Default)]
struct AuraParams {
    first: Vec4,
    second: Vec4,
    /// How far it spreads, as a part of the width and height.
    reach: Vec2,
    /// 1 to glow inside the mask, 0 outside it.
    inward: f32,
    time: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(super) struct AuraMaterial {
    #[uniform(0)]
    params: AuraParams,
    #[texture(1)]
    #[sampler(2)]
    mask: Handle<Image>,
}

impl UiMaterial for AuraMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/item_aura.wesl".into()
    }
}

/// What has been made already, as few things glow and many look the same.
#[derive(Resource, Default)]
pub(super) struct Auras {
    masks: HashMap<Shape, Handle<Image>>,
    materials: HashMap<(AssetId<Image>, Aura), Handle<AuraMaterial>>,
}

/// An image of these cells: solid where it has one.
fn shape_mask(shape: Shape) -> Image {
    let (width, height) = (shape.width as usize, shape.height as usize);
    let (across, down) = (width * MASK_PIXELS_PER_CELL, height * MASK_PIXELS_PER_CELL);
    let mut data = vec![0; across * down * 4];
    for (x, y) in shape.cells() {
        for row in 0..MASK_PIXELS_PER_CELL {
            for column in 0..MASK_PIXELS_PER_CELL {
                let pixel_y = y as usize * MASK_PIXELS_PER_CELL + row;
                let pixel_x = x as usize * MASK_PIXELS_PER_CELL + column;
                let pixel = (pixel_y * across + pixel_x) * 4;
                data[pixel..pixel + 4].copy_from_slice(&[255; 4]);
            }
        }
    }
    Image::new(
        Extent3d {
            width: across as u32,
            height: down as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

pub(super) fn give_auras(
    new: Query<(Entity, &AuraGlow), Added<AuraGlow>>,
    mut auras: ResMut<Auras>,
    mut pictures: ResMut<ItemPictures>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<AuraMaterial>>,
    mut commands: Commands,
) {
    let auras = &mut *auras;
    let reach_of = |cells: Vec2| Vec2::splat(SPREAD) / (cells * CELL);
    for (entity, glow) in &new {
        let (mask, reach, inward) = match &glow.mask {
            AuraMask::Picture(look, pixels) => {
                let picture = pictures.picture(look.clone(), *pixels, &mut images);
                let cells = *pixels as f32 / PICTURE_PIXELS_PER_CELL as f32;
                (picture, reach_of(Vec2::splat(cells)), 0.0)
            }
            AuraMask::Cells(shape) => {
                let mask = auras
                    .masks
                    .entry(*shape)
                    .or_insert_with(|| images.add(shape_mask(*shape)))
                    .clone();
                let cells = Vec2::new(shape.width as f32, shape.height as f32);
                (mask, reach_of(cells), 1.0)
            }
        };
        let material = auras
            .materials
            .entry((mask.id(), glow.aura))
            .or_insert_with(|| {
                let (first, second) = glow.aura.colours();
                materials.add(AuraMaterial {
                    params: AuraParams {
                        first: LinearRgba::from(first).to_vec4(),
                        second: LinearRgba::from(second).to_vec4(),
                        reach,
                        inward,
                        time: 0.0,
                    },
                    mask,
                })
            })
            .clone();
        commands.entity(entity).insert(MaterialNode(material));
    }
}

/// Keeps them pulsing, while any are there.
pub(super) fn pulse_auras(
    time: Res<Time<Real>>,
    shown: Query<(), With<AuraGlow>>,
    auras: Res<Auras>,
    mut materials: ResMut<Assets<AuraMaterial>>,
) {
    if shown.is_empty() {
        return;
    }
    for handle in auras.materials.values() {
        if let Some(mut material) = materials.get_mut(handle) {
            material.params.time = time.elapsed_secs();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::id::Id;
    use shared::item::{ItemKind, Made, Quality};

    /// Bevy only finds two parameters clashing when a system starts.
    #[test]
    fn no_system_asks_for_what_another_of_its_parameters_has() {
        IntoSystem::into_system(give_auras).initialize(&mut World::new());
        IntoSystem::into_system(pulse_auras).initialize(&mut World::new());
    }

    #[test]
    fn only_what_is_known_to_be_special_glows() {
        let plain = Item::new(Id::for_test(1), ItemKind::Hat);
        assert_eq!(Aura::of(&plain), None);
        let magical = Item {
            made: Made {
                magical: true,
                ..plain.made
            },
            quality: Some(Quality::Rare),
            ..plain.clone()
        };
        assert_eq!(Aura::of(&magical).unwrap().colours(), (MAGICAL, MAGICAL));
        let both = Item {
            favour: Some(Favour::Cursed),
            ..magical
        };
        assert_eq!(Aura::of(&both).unwrap().colours(), (MAGICAL, CURSED));
        let blessed = Item {
            favour: Some(Favour::Blessed),
            ..plain
        };
        assert_eq!(Aura::of(&blessed).unwrap().colours(), (BLESSED, BLESSED));
    }

    #[test]
    fn a_mask_is_solid_only_on_its_cells() {
        let image = shape_mask(Shape::rectangle(2, 1));
        let data = image.data.unwrap();
        assert_eq!(data.len(), 16 * 8 * 4);
        assert!(data.iter().all(|&byte| byte == 255));
    }
}
