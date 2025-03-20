use crate::tools::{Intensity, IntensityChangePerSecond, Units};
use bevy::prelude::*;

pub trait HandlesLights {
	type TResponsiveLightBundle: Bundle;
	type TResponsiveLightTrigger: Bundle;

	fn responsive_light_trigger() -> Self::TResponsiveLightTrigger;
	fn responsive_light_bundle<TShareMaterials>(
		responsive_light: Responsive,
	) -> Self::TResponsiveLightBundle
	where
		TShareMaterials: 'static;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Responsive {
	pub light: Light,
	pub range: Units,
	pub light_on_material: fn() -> StandardMaterial,
	pub light_off_material: fn() -> StandardMaterial,
	pub max: Intensity,
	pub change: IntensityChangePerSecond,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Light {
	Point(fn() -> PointLight),
	Spot(fn() -> SpotLight),
	Directional(fn() -> DirectionalLight),
}
