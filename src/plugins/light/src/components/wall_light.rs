use crate::observers::get_insert_system::TargetMeshName;
use bevy::prelude::*;
use zyheeda_core::prelude::NormalizedNameLazy;

#[derive(Component, Debug, PartialEq, Default)]
#[require(SpotLight = Self)]
pub(crate) struct WallLight;

impl From<WallLight> for SpotLight {
	fn from(_: WallLight) -> Self {
		SpotLight {
			range: 30.0,
			inner_angle: 45_f32.to_radians(),
			outer_angle: 80_f32.to_radians(),
			shadows_enabled: false,
			..default()
		}
	}
}

impl TargetMeshName for WallLight {
	fn target_mesh_name() -> NormalizedNameLazy {
		const { NormalizedNameLazy::from_name("WallLight") }
	}
}
