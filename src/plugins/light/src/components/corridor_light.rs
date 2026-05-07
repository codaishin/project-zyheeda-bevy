use crate::{components::light::Light, observers::get_insert_system::TargetMeshName};
use bevy::{color::palettes::css::ANTIQUE_WHITE, prelude::*};
use zyheeda_core::prelude::NormalizedNameLazy;

#[derive(Component, Debug, PartialEq, Default)]
#[require(Light, SpotLight = Self)]
pub(crate) struct CorridorLight;

impl From<CorridorLight> for SpotLight {
	fn from(_: CorridorLight) -> Self {
		SpotLight {
			range: 6.0,
			radius: 0.2,
			intensity: 100_000.0,
			inner_angle: f32::to_radians(0.),
			outer_angle: f32::to_radians(89.),
			shadows_enabled: false,
			color: ANTIQUE_WHITE.into(),
			..default()
		}
	}
}

impl TargetMeshName for CorridorLight {
	fn target_mesh_name() -> NormalizedNameLazy {
		const { NormalizedNameLazy::from_name("CorridorLight") }
	}
}
