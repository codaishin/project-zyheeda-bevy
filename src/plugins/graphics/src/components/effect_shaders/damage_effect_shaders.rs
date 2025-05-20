use crate::components::camera_labels::SecondPass;
use bevy::{color::palettes::css::WHITE, prelude::*, render::view::RenderLayers};
use common::components::spawn_children::SpawnChildren;

#[derive(Component, Debug, PartialEq, Default)]
#[require(SpawnChildren = Self::children())]
pub struct DamageEffectShaders;

impl DamageEffectShaders {
	fn children() -> SpawnChildren {
		SpawnChildren(|parent| {
			parent.spawn((
				RenderLayers::from(SecondPass),
				PointLight {
					color: Color::from(WHITE),
					intensity: 8000.,
					shadows_enabled: true,
					..default()
				},
			));
		})
	}
}
