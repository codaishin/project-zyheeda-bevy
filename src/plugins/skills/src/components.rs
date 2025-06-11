pub mod combo_node;
pub mod combos;
pub mod combos_time_out;
pub mod inventory;
pub mod model_render;
pub mod queue;
pub mod slots;
pub mod swapper;

pub(crate) mod skill_executer;

use bevy::prelude::*;
use common::{components::outdated::Outdated, tools::collider_info::ColliderInfo};

pub type SkillTarget = SelectInfo<Outdated<GlobalTransform>>;

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
