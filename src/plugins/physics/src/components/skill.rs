mod contact;
mod dto;
mod lifetime;
mod motion;
mod projection;

use crate::components::{
	collider::ColliderShape,
	interaction_target::InteractionTarget,
	skill::dto::SkillDto,
};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	tools::{Units, UnitsPerSecond},
	traits::handles_skill_physics::{Effect, SkillCaster, SkillShape, SkillSpawner, SkillTarget},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[require(PersistentEntity, Transform, Visibility)]
#[savable_component(dto = SkillDto)]
pub struct Skill {
	pub(crate) created_from: CreatedFrom,
	pub(crate) shape: SkillShape,
	pub(crate) contact_effects: Vec<Effect>,
	pub(crate) projection_effects: Vec<Effect>,
	pub(crate) caster: SkillCaster,
	pub(crate) spawner: SkillSpawner,
	pub(crate) target: SkillTarget,
}

#[derive(Component, Debug, PartialEq)]
#[require(InteractionTarget, Transform, Visibility)]
pub struct ContactInteractionTarget;

#[derive(Component, Debug, PartialEq)]
#[require(InteractionTarget, Transform, Visibility)]
pub struct ProjectionInteractionTarget;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum CreatedFrom {
	Spawn,
	Save,
}

const SPHERE_MODEL: &str = "models/sphere.glb";

const BEAM_MODEL: fn() -> Mesh = || {
	Mesh::from(Cylinder {
		radius: 1.,
		half_height: 0.5,
	})
};
const BEAM_CONTACT_RADIUS: f32 = 0.003;
const BEAM_PROJECTION_RADIUS: f32 = 0.2;

const PROJECTILE_CONTACT_RADIUS: f32 = 0.05;
const PROJECTILE_PROJECTION_RADIUS: f32 = 0.5;
const PROJECTILE_RANGE: Units = Units::from_u8(20);
const PROJECTILE_SPEED: UnitsPerSecond = UnitsPerSecond::from_u8(15);

const HALF_FORWARD: Transform = Transform::from_translation(Vec3 {
	x: 0.,
	y: 0.,
	z: -0.5,
});
static HOLLOW_OUTER_THICKNESS: LazyLock<Units> = LazyLock::new(|| Units::from(0.3));

const SHIELD_MODEL: &str = "models/shield.glb";
static SHIELD_CONTACT_COLLIDER: LazyLock<ColliderShape> = LazyLock::new(|| ColliderShape::Cuboid {
	half_x: Units::from(0.5),
	half_y: Units::from(0.5),
	half_z: Units::from(0.05),
});
const SHIELD_PROJECTION_SCALE: Vec3 = Vec3 {
	x: 2.,
	y: 2.,
	z: 2.,
};

const ICO_SPHERE_HALF: &str = "models/icosphere_half.glb#Mesh0/Primitive0";
