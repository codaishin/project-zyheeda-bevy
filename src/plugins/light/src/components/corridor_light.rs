use crate::{components::light::Light, observers::get_insert_system::TargetMeshName};
use bevy::{color::palettes::css::ANTIQUE_WHITE, prelude::*};
use common::components::insert_asset::InsertAsset;
use zyheeda_core::prelude::NormalizedNameLazy;

#[derive(Component, Debug, PartialEq, Default)]
#[require(Light, SpotLight = Self, InsertAsset<StandardMaterial> = Self)]
pub(crate) struct CorridorLight;

impl CorridorLight {
	const COLOR: Srgba = ANTIQUE_WHITE;
}

impl From<CorridorLight> for SpotLight {
	fn from(_: CorridorLight) -> Self {
		SpotLight {
			range: 6.0,
			radius: 0.2,
			intensity: 100_000.0,
			inner_angle: f32::to_radians(0.),
			outer_angle: f32::to_radians(89.),
			shadows_enabled: false,
			color: CorridorLight::COLOR.into(),
			..default()
		}
	}
}

impl From<CorridorLight> for InsertAsset<StandardMaterial> {
	fn from(_: CorridorLight) -> Self {
		InsertAsset::shared::<CorridorLight>(|| StandardMaterial {
			base_color: CorridorLight::COLOR.into(),
			emissive: (CorridorLight::COLOR * 5.).into(),
			..default()
		})
	}
}

impl TargetMeshName for CorridorLight {
	fn target_mesh_name() -> NormalizedNameLazy {
		const { NormalizedNameLazy::from_name("CorridorLight") }
	}
}
