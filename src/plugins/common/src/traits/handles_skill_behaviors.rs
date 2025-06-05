use crate::{
	blocker::Blocker,
	components::{AssetModel, persistent_entity::PersistentEntity},
	tools::{Units, UnitsPerSecond, action_key::slot::SlotKey},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub trait HandlesSkillBehaviors {
	type TSkillContact: Component;
	type TSkillProjection: Component;

	fn spawn_skill(
		commands: &mut Commands,
		contact: Contact,
		projection: Projection,
	) -> SkillEntities;
}

pub struct Contact {
	pub shape: Shape,
	pub integrity: Integrity,
	pub motion: Motion,
}

pub struct Projection {
	pub shape: Shape,
	pub offset: Option<ProjectionOffset>,
}

pub struct SkillEntities {
	pub root: Entity,
	pub contact: Entity,
	pub projection: Entity,
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
		caster: PersistentEntity,
	},
	Stationary {
		caster: PersistentEntity,
		max_cast_range: Units,
		target_ray: Ray3d,
	},
	Projectile {
		caster: PersistentEntity,
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
