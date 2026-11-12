use crate::components::{
	blockable::Blockable,
	character_motion::IsInMotion,
	collider::{AGENTS_GROUP, ColliderShape, RAY_GROUP, SKILLS_GROUP},
	collision_domains::Physical,
	effects::Effects,
	impact_able::ImpactAble,
	model::PhysicsModel,
	motion_controller::{MotionBehavior, MotionControllerOf},
	persistent_root::PersistentRoot,
	self_skill_scale::SelfSkillScale,
	skill::{VisualSkillContact, VisualSkillProjection},
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

		let (cont_model, cont_effects, proj_model, proj_effects, controller) = physical(
			&mut commands,
			root,
			skill,
			persistent_entity,
			self_skill_scale,
		);

		let Some(entity) = commands.get_mut(&root) else {
			return Err(SkillSpawnError::EntityMissing(root));
		};

		visual(
			entity,
			controller,
			self_skill_scale,
			cont_model,
			cont_effects,
			proj_model,
			proj_effects,
		);

		Ok(())
	}
}

fn physical(
	commands: &mut ZyheedaCommands<'_, '_>,
	root: Entity,
	skill: &impl SkillPrefab,
	persistent_entity: &PersistentEntity,
	self_skill_scale: &SelfSkillScale,
) -> (SubModel, Effects, SubModel, Effects, Entity) {
	let (obj, cont_model, cont_collider, cont_effects) = skill.get_contact_prefab(self_skill_scale);
	let (proj_model, proj_collider, proj_effects) = skill.get_projection_prefab(self_skill_scale);
	let collision_group = CollisionGroups {
		memberships: SKILLS_GROUP,
		filters: SKILLS_GROUP | AGENTS_GROUP | RAY_GROUP,
		test_mode: InteractionTestMode::And,
	};

	let mut controller = commands.spawn((
		#[cfg(feature = "debug-utils")]
		Name::from("Skill Physical"),
		*self_skill_scale,
	));

	controller.insert((
		Blockable::from(obj).ignoring(skill.get_caster()),
		PersistentRoot(*persistent_entity),
		ImpactAble,
		cont_effects.clone(),
		children![
			(
				SkillTransformOf(controller.id()),
				collision_group,
				cont_collider.transform,
				cont_collider.shape,
				Physical::Contact,
				Sensor,
			),
			(
				PersistentRoot(*persistent_entity),
				Transform::default(),
				proj_effects.clone(),
				children![(
					SkillTransformOf(controller.id()),
					collision_group,
					proj_collider.transform,
					proj_collider.shape,
					Physical::Projection,
					Sensor,
				)]
			)
		],
	));

	let movement_behavior = skill.apply_motion_prefab(&mut controller);

	controller.try_insert(MotionControllerOf::skill(root).with_behavior(movement_behavior));

	if let Some(lifetime) = skill.get_lifetime() {
		controller.try_insert(Lifetime::from(lifetime));
	};

	(
		cont_model,
		cont_effects,
		proj_model,
		proj_effects,
		controller.id(),
	)
}

fn visual(
	mut entity: ZyheedaEntityCommands<'_>,
	controller: Entity,
	self_skill_scale: &SelfSkillScale,
	cont_model: SubModel,
	cont_effects: Effects,
	proj_model: SubModel,
	proj_effects: Effects,
) {
	entity.try_insert((
		#[cfg(feature = "debug-utils")]
		Name::from("Skill Visuals"),
		*self_skill_scale,
		IsInMotion,
		Visibility::Visible,
		Transform::default(),
		VisualSkillContact,
		cont_effects,
		children![
			(
				SkillTransformOf(controller),
				cont_model.transform,
				cont_model.model,
			),
			(
				Visibility::Visible,
				VisualSkillProjection,
				Transform::default(),
				proj_effects,
				children![(
					SkillTransformOf(controller),
					proj_model.transform,
					proj_model.model,
				)]
			)
		],
	));
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
	fn apply_motion_prefab(&self, entity: &mut EntityCommands) -> MotionBehavior;
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
