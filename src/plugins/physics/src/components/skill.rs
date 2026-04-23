mod contact;
mod dto;
mod lifetime;
mod motion;
mod projection;

use crate::components::{interaction_target::InteractionTarget, skill::dto::SkillDto};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	tools::{Units, UnitsPerSecond},
	traits::handles_skill_physics::{Effect, SkillCaster, SkillMount, SkillShape},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[require(PersistentEntity, Transform, Visibility)]
#[savable_component(id = "skill", dto = SkillDto)]
pub struct Skill {
	pub(crate) created_from: CreatedFrom,
	pub(crate) shape: SkillShape,
	pub(crate) contact_effects: Vec<Effect>,
	pub(crate) projection_effects: Vec<Effect>,
	pub(crate) caster: SkillCaster,
	pub(crate) mount: SkillMount,
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

const SHIELD_CONTACT_MODEL: &str = "models/shield/contact.glb";
const SHIELD_CONTACT_COLLIDER: &str = "models/shield/contact_collider.glb#Mesh0/Primitive0";
const SHIELD_PROJECTION_MODEL: &str = "models/shield/projection.glb";
const SHIELD_PROJECTION_COLLIDER: &str = "models/shield/projection_collider.glb#Mesh0/Primitive0";

//FIXME: Should not be necessary, see: https://github.com/codaishin/project-zyheeda-bevy/issues/761
const SHIELD_SCALE: Vec3 = Vec3 {
	x: 1.7,
	y: 1.7,
	z: 1.7,
};
