use crate::components::skill_prefabs::skill_contact::CreatedFrom;
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::handles_skill_physics::{Contact, Effect, Projection},
};
use std::time::Duration;

#[derive(Component, Debug, PartialEq, Clone)]
#[require(PersistentEntity)]
pub struct Skill {
	pub(crate) lifetime: Option<Duration>,
	pub(crate) created_from: CreatedFrom,
	pub(crate) contact: Contact,
	pub(crate) contact_effects: Vec<Effect>,
	pub(crate) projection: Projection,
	pub(crate) projection_effects: Vec<Effect>,
}

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
