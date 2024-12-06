use crate::tools::{Intensity, IntensityChangePerSecond, Units};
use bevy::prelude::*;

pub trait HandlesLights {
	type TResponsiveLightBundle: Bundle;

	fn responsive_light_bundle(responsive_light: Responsive) -> Self::TResponsiveLightBundle;
}

#[derive(Debug, PartialEq, Clone)]
pub struct Responsive {
	pub model: Entity,
	pub light: Entity,
	pub range: Units,
	pub light_on_material: Handle<StandardMaterial>,
	pub light_off_material: Handle<StandardMaterial>,
	pub max: Intensity,
	pub change: IntensityChangePerSecond,
}
