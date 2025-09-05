pub(crate) mod combo_node;
pub(crate) mod combos;
pub(crate) mod combos_time_out;
pub(crate) mod inventory;
pub(crate) mod loadout;
pub(crate) mod model_render;
pub(crate) mod queue;
pub(crate) mod skill_executer;
pub(crate) mod slots;

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
