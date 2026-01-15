mod contact;
mod dto;
mod lifetime;
mod motion;
mod projection;

use crate::components::{interaction_target::InteractionTarget, skill::dto::SkillDto};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	tools::Units,
	traits::handles_skill_physics::{Contact, Effect, Projection},
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::{sync::LazyLock, time::Duration};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[require(PersistentEntity, Transform, Visibility)]
#[savable_component(dto = SkillDto)]
pub struct Skill {
	pub(crate) lifetime: Option<Duration>,
	pub(crate) created_from: CreatedFrom,
	pub(crate) contact: Contact,
	pub(crate) contact_effects: Vec<Effect>,
	pub(crate) projection: Projection,
	pub(crate) projection_effects: Vec<Effect>,
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
const HALF_FORWARD: Transform = Transform::from_translation(Vec3 {
	x: 0.,
	y: 0.,
	z: -0.5,
});
static HOLLOW_OUTER_THICKNESS: LazyLock<Units> = LazyLock::new(|| Units::from(0.3));

struct Beam;

#[cfg(test)]
mod test_impls {
	use super::*;
	use common::{
		tools::Units,
		traits::handles_skill_physics::{
			ContactShape,
			Motion,
			ProjectionShape,
			SkillCaster,
			SkillTarget,
		},
	};
	use std::collections::HashSet;

	impl Default for Skill {
		fn default() -> Self {
			Self {
				lifetime: None,
				created_from: CreatedFrom::Spawn,
				contact: Contact {
					shape: ContactShape::Beam {
						range: Units::from_u8(10),
						radius: Units::from_u8(1),
						blocked_by: HashSet::from([]),
					},
					motion: Motion::Stationary {
						caster: SkillCaster(PersistentEntity::default()),
						max_cast_range: Units::from_u8(1),
						target: SkillTarget::Ground(Vec3::ZERO),
					},
				},
				contact_effects: vec![],
				projection: Projection {
					shape: ProjectionShape::Beam {
						radius: Units::from_u8(2),
					},
					offset: None,
				},
				projection_effects: vec![],
			}
		}
	}
}
