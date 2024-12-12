pub(crate) mod skill_contact;

use bevy::prelude::*;
use bevy_rapier3d::prelude::Collider;
use common::{
	blocker::Blocker,
	components::{AssetModel, Outdated},
	resources::ColliderInfo,
	tools::{Units, UnitsPerSecond},
};

#[derive(Debug, Clone)]
pub enum Shape {
	Sphere {
		radius: Units,
	},
	Custom {
		model: AssetModel,
		collider: Collider,
		scale: Vec3,
	},
}

#[derive(Debug, PartialEq, Clone)]
pub enum Integrity {
	Solid,
	Fragile { destroyed_by: Vec<Blocker> },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Motion {
	HeldBy {
		spawner: Entity,
	},
	Stationary {
		caster: Entity,
		max_cast_range: Units,
		target_ray: Ray3d,
	},
	Projectile {
		caster: Entity,
		spawner: Entity,
		speed: UnitsPerSecond,
		max_range: Units,
	},
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SelectInfo<T> {
	pub ray: Ray3d,
	pub collision_info: Option<ColliderInfo<T>>,
}

impl<T> Default for SelectInfo<T> {
	fn default() -> Self {
		Self {
			ray: Ray3d {
				origin: Vec3::ZERO,
				direction: Dir3::NEG_Z,
			},
			collision_info: None,
		}
	}
}

pub type SkillTarget = SelectInfo<Outdated<GlobalTransform>>;

impl From<Ray3d> for SkillTarget {
	fn from(ray: Ray3d) -> Self {
		Self { ray, ..default() }
	}
}

impl SkillTarget {
	pub fn with_ray(self, ray: Ray3d) -> Self {
		Self {
			ray,
			collision_info: self.collision_info,
		}
	}

	pub fn with_collision_info(
		self,
		collision_info: Option<ColliderInfo<Outdated<GlobalTransform>>>,
	) -> Self {
		Self {
			ray: self.ray,
			collision_info,
		}
	}
}
