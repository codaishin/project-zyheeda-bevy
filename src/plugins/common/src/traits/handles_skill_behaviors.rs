use crate::{
	blocker::Blocker,
	components::{asset_model::AssetModel, persistent_entity::PersistentEntity},
	tools::{Units, UnitsPerSecond, action_key::slot::SlotKey},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub trait HandlesSkillBehaviors {
	type TSkillContact: Component;
	type TSkillProjection: Component;

	fn spawn_skill(
		commands: &mut Commands,
		contact: Contact,
		projection: Projection,
	) -> SkillEntities;
}

#[derive(Debug, Clone)]
pub struct Contact {
	pub shape: Shape,
	pub integrity: Integrity,
	pub motion: Motion,
}

#[derive(Debug, Clone)]
pub struct Projection {
	pub shape: Shape,
	pub offset: Option<ProjectionOffset>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SkillEntities {
	pub root: SkillRoot,
	pub contact: Entity,
	pub projection: Entity,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SkillRoot {
	pub entity: Entity,
	pub persistent_entity: PersistentEntity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Integrity {
	Solid,
	Fragile { destroyed_by: HashSet<Blocker> },
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
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
		range: Units,
	},
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Spawner {
	#[default]
	Center,
	Slot(SlotKey),
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct ProjectionOffset(pub Vec3);
