mod contact;
mod dto;
mod lifetime;
mod motion;
mod projection;

use crate::components::{markers::Physical, skill::dto::SkillDto};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	tools::{Units, UnitsPerSecond},
	traits::handles_skill_physics::{Effect, SkillCaster, SkillMount, SkillShape},
};
use macros::{SavableComponent, asset_path};
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
#[require(Physical, Transform, Visibility)]
pub struct SkillContactRoot;

#[derive(Component, Debug, PartialEq)]
#[require(Physical, Transform, Visibility)]
pub struct SkillProjectionRoot;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum CreatedFrom {
	Spawn,
	Save,
}

const SPHERE_MODEL: &str = asset_path!("generic/models/sphere.glb");

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

macro_rules! shield_path {
	(asset($asset:literal)) => {
		asset_path!("items/force_essence/skills/shield", $asset, ".glb")
	};
	(mesh($asset:literal)) => {
		concat!(shield_path!(asset($asset)), "#Mesh0/Primitive0")
	};
}

const SHIELD_CONTACT_MODEL: &str = shield_path!(asset("contact"));
const SHIELD_CONTACT_COLLIDER: &str = shield_path!(mesh("contact_collider"));
const SHIELD_PROJECTION_MODEL: &str = shield_path!(asset("projection"));
const SHIELD_PROJECTION_COLLIDER: &str = shield_path!(mesh("projection_collider"));

//FIXME: Should not be necessary, see: https://github.com/codaishin/project-zyheeda-bevy/issues/761
const SHIELD_SCALE: Vec3 = Vec3 {
	x: 1.7,
	y: 1.7,
	z: 1.7,
};
