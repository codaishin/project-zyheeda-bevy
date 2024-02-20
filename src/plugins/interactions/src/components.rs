use bevy::{ecs::component::Component, math::Vec3};
use bevy_rapier3d::pipeline::QueryFilter;
use common::traits::cast_ray::TimeOfImpact;

#[derive(Component, Debug, PartialEq, Clone)]
pub struct RayCaster {
	pub origin: Vec3,
	pub direction: Vec3,
	pub max_toi: TimeOfImpact,
	pub solid: bool,
	pub get_filter: fn() -> QueryFilter<'static>,
}

impl Default for RayCaster {
	fn default() -> Self {
		Self {
			origin: Default::default(),
			direction: Default::default(),
			max_toi: Default::default(),
			solid: Default::default(),
			get_filter: Default::default,
		}
	}
}

#[derive(Component)]
pub(crate) struct Destroy;

#[derive(Component)]
pub struct DealsDamage(pub i16);
