use crate::tools::{Intensity, IntensityChangePerSecond, Units};
use bevy::prelude::*;

pub trait HandlesLights {
	type TResponsiveLightBundle: Bundle;
	type TResponsiveLightTrigger: Bundle;

	fn responsive_light_trigger() -> Self::TResponsiveLightTrigger;
	fn responsive_light_bundle<TMarker>(
		responsive_light: Responsive,
	) -> Self::TResponsiveLightBundle
	where
		TMarker: 'static;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Responsive {
	pub model: Entity,
	pub light: Entity,
	pub range: Units,
	pub light_on_material: fn() -> StandardMaterial,
	pub light_off_material: fn() -> StandardMaterial,
	pub max: Intensity,
	pub change: IntensityChangePerSecond,
}
