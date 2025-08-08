use crate::tools::{Intensity, IntensityChangePerSecond, Units};
use bevy::prelude::*;

pub trait HandlesLights {
	type TResponsiveLightBundle: Bundle;
	type TResponsiveLightTrigger: Bundle;

	const DEFAULT_LIGHT: Srgba;

	fn responsive_light_trigger() -> Self::TResponsiveLightTrigger;
	fn responsive_light_bundle<TShareMaterials>(
		responsive_light: Responsive,
	) -> Self::TResponsiveLightBundle
	where
		TShareMaterials: 'static;
}

#[derive(Debug, Clone)]
pub struct Responsive {
	pub light: Light,
	pub range: Units,
	pub light_on_material: fn() -> StandardMaterial,
	pub light_off_material: fn() -> StandardMaterial,
	pub max: Intensity,
	pub change: IntensityChangePerSecond,
}

impl PartialEq for Responsive {
	fn eq(&self, other: &Self) -> bool {
		self.light == other.light
			&& self.range == other.range
			&& self.max == other.max
			&& self.change == other.change
			&& std::ptr::fn_addr_eq(self.light_on_material, other.light_on_material)
			&& std::ptr::fn_addr_eq(self.light_off_material, other.light_off_material)
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Light {
	Point(fn() -> PointLight),
	Spot(fn() -> SpotLight),
	Directional(fn() -> DirectionalLight),
}

impl PartialEq for Light {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Point(l0), Self::Point(r0)) => std::ptr::fn_addr_eq(*l0, *r0),
			(Self::Spot(l0), Self::Spot(r0)) => std::ptr::fn_addr_eq(*l0, *r0),
			(Self::Directional(l0), Self::Directional(r0)) => std::ptr::fn_addr_eq(*l0, *r0),
			_ => false,
		}
	}
}
