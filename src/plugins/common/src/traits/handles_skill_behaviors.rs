use crate::{
	blocker::Blocker,
	components::AssetModel,
	tools::{Units, UnitsPerSecond, action_key::slot::SlotKey},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub trait HandlesSkillBehaviors {
	type TSkillContact: Component;
	type TSkillProjection: Component;

	fn skill_contact(shape: Shape, integrity: Integrity, motion: Motion) -> Self::TSkillContact;
	fn skill_projection(shape: Shape, offset: Option<ProjectionOffset>) -> Self::TSkillProjection;
}

#[derive(Debug, Clone)]
pub enum Shape {
	Sphere {
		radius: Units,
		hollow_collider: bool,
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
		caster: Entity,
	},
	Stationary {
		caster: Entity,
		max_cast_range: Units,
		target_ray: Ray3d,
	},
	Projectile {
		caster: Entity,
		spawner: Spawner,
		speed: UnitsPerSecond,
		max_range: Units,
	},
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub enum Spawner {
	#[default]
	Center,
	Slot(SlotKey),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ProjectionOffset(pub Vec3);
