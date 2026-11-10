use crate::components::{
	blockable::Blockable,
	character_motion::IsInMotion,
	collider::{AGENTS_GROUP, ColliderShape, RAY_GROUP, SKILLS_GROUP},
	collision_domains::Physical,
	effects::Effects,
	model::PhysicsModel,
	motion_controller::MotionControllerOf,
	persistent_root::PersistentRoot,
	self_skill_scale::SelfSkillScale,
	skill::{SkillContactRoot, SkillProjectionRoot},
	skill_transform::SkillTransformOf,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::prelude::*;
use std::{fmt::Display, time::Duration};

impl<T> SkillPrefab for T where
	T: Component
		+ GetCaster
		+ GetLifetime
		+ GetContactPrefab
		+ GetProjectionPrefab
		+ ApplyMotionPrefab
{
}

pub(crate) trait SkillPrefab:
	Component
	+ GetCaster
	+ GetLifetime
	+ GetContactPrefab
	+ GetProjectionPrefab
	+ ApplyMotionPrefab
	+ Sized
{
	fn prefab(
		on_insert: On<Insert<Self>>,
		mut commands: ZyheedaCommands,
		casters: Query<&SelfSkillScale>,
		skills: Query<(&Self, &PersistentEntity, Option<&SelfSkillScale>)>,
	) -> Result<(), SkillSpawnError> {
		let root = on_insert.entity;
		let Ok((skill, persistent_entity, cached_scale)) = skills.get(root) else {
			return Ok(());
		};
		let scale_from_caster = || {
			commands
				.get(&skill.get_caster())
				.and_then(|e| casters.get(e).ok())
		};
		let Some(self_skill_scale) = get_scale(cached_scale, scale_from_caster) else {
			return Err(SkillSpawnError::SelfSkillScaleMissing(*persistent_entity));
		};
		let Some(mut entity) = commands.get_mut(&root) else {
			return Err(SkillSpawnError::EntityMissing(root));
		};
		let (obj, cont_model, cont_collider, cont_effects) =
			skill.get_contact_prefab(self_skill_scale);
		let (proj_model, proj_collider, proj_effects) =
			skill.get_projection_prefab(self_skill_scale);
		let collision_group = CollisionGroups {
			memberships: SKILLS_GROUP,
			filters: SKILLS_GROUP | AGENTS_GROUP | RAY_GROUP,
			test_mode: InteractionTestMode::And,
		};

		entity.try_insert((
			#[cfg(feature = "debug-utils")]
			Name::from("Skill Visuals"),
			IsInMotion,
			Visibility::Visible,
			Transform::default(),
			*self_skill_scale,
			children![
				(
					Visibility::Visible,
					SkillContactRoot,
					SkillTransformOf(root),
					cont_effects.clone(),
					cont_model.transform,
					cont_model.model,
				),
				(
					Visibility::Visible,
					SkillProjectionRoot,
					SkillTransformOf(root),
					proj_effects.clone(),
					proj_model.transform,
					proj_model.model
				)
			],
		));

		let mut controller = commands.spawn((
			MotionControllerOf::entity(root),
			PersistentRoot(*persistent_entity),
			Blockable(obj),
			cont_effects,
			*self_skill_scale,
			children![
				(
					SkillTransformOf(root),
					collision_group,
					cont_collider.transform,
					cont_collider.shape,
					Physical::Contact,
					Sensor,
				),
				(
					PersistentRoot(*persistent_entity),
					Transform::default(),
					proj_effects,
					children![(
						SkillTransformOf(root),
						collision_group,
						proj_collider.transform,
						proj_collider.shape,
						Physical::Projection,
						Sensor,
					)]
				)
			],
		));

		skill.apply_motion_prefab(&mut controller);

		if let Some(lifetime) = skill.get_lifetime() {
			controller.try_insert(Lifetime::from(lifetime));
		};

		Ok(())
	}
}

fn get_scale<'a>(
	cached: Option<&'a SelfSkillScale>,
	from_caster: impl Fn() -> Option<&'a SelfSkillScale>,
) -> Option<&'a SelfSkillScale> {
	match cached {
		Some(scale) => Some(scale),
		None => from_caster(),
	}
}

pub(crate) trait GetCaster {
	fn get_caster(&self) -> PersistentEntity;
}

pub(crate) trait GetLifetime {
	fn get_lifetime(&self) -> Option<Duration>;
}

pub(crate) trait GetContactPrefab {
	fn get_contact_prefab(
		&self,
		scale: &SelfSkillScale,
	) -> (PhysicalObject, SubModel, ContactCollider, Effects);
}

pub(crate) trait GetProjectionPrefab {
	fn get_projection_prefab(
		&self,
		scale: &SelfSkillScale,
	) -> (SubModel, ProjectionCollider, Effects);
}

pub(crate) trait ApplyMotionPrefab {
	fn apply_motion_prefab(&self, entity: &mut EntityCommands);
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SubModel {
	pub(crate) model: PhysicsModel,
	pub(crate) transform: Transform,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ProjectionCollider {
	pub(crate) shape: ColliderShape,
	pub(crate) transform: Transform,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ContactCollider {
	pub(crate) shape: ColliderShape,
	pub(crate) transform: Transform,
}

#[derive(Debug, PartialEq)]
pub(crate) enum SkillSpawnError {
	EntityMissing(Entity),
	SelfSkillScaleMissing(PersistentEntity),
}

impl Display for SkillSpawnError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use SkillSpawnError::*;

		match self {
			EntityMissing(e) => write!(f, "{e}: missing"),
			SelfSkillScaleMissing(e) => write!(f, "{e:?}: missing `SelfSkillScale`"),
		}
	}
}

impl ErrorData for SkillSpawnError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl Display {
		"Skill spawn error"
	}

	fn into_details(self) -> impl Display {
		self
	}
}
