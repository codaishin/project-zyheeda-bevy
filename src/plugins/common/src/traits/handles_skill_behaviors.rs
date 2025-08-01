use crate::{
	components::{
		asset_model::AssetModel,
		is_blocker::Blocker,
		persistent_entity::PersistentEntity,
	},
	tools::{Units, UnitsPerSecond, action_key::slot::SlotKey},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub trait HandlesSkillBehaviors {
	type TSkillContact: Component;
	type TSkillProjection: Component;

	/// Skills always have a contact and a projection shape.
	///
	/// Activity of those shapes should controlled by their attached effects.
	fn spawn_skill(
		commands: &mut Commands,
		contact: Contact,
		projection: Projection,
	) -> SkillEntities;
}

/// Describes the contact shape of a skill
///
/// These should be used for physical effects like projectile bodies, barriers or beam cores.
#[derive(Debug, Clone)]
pub struct Contact {
	pub shape: Shape,
	pub motion: Motion,
}

/// Describes the projection shape of a skill
///
/// These should be used for AoE.
#[derive(Debug, Clone)]
pub struct Projection {
	pub shape: Shape,
	pub offset: Option<ProjectionOffset>,
}

/// The entities of a spawned skill.
///
/// Skill root should be used to control the lifetime of a skill and can be used to despawn the
/// whole skill.
///
/// Contact components should be added to the contact entity.
///
/// Projection/AoE components should be added to the projection entity.
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
		destroyed_by: HashSet<Blocker>,
	},
	Custom {
		model: AssetModel,
		collider: Collider,
		scale: Vec3,
		destroyed_by: HashSet<Blocker>,
	},
	Beam {
		range: Units,
		radius: Units,
		blocked_by: HashSet<Blocker>,
	},
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Motion {
	HeldBy {
		caster: PersistentEntity,
		spawner: Spawner,
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
