use crate::components::skill_prefabs::skill_contact::CreatedFrom;
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::handles_skill_physics::{Contact, Projection},
};

#[derive(Component, Debug, PartialEq)]
#[require(PersistentEntity)]
pub struct Skill {
	pub(crate) created_from: CreatedFrom,
	pub(crate) contact: Contact,
	pub(crate) projection: Projection,
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
				projection: Projection {
					shape: ProjectionShape::Beam {
						radius: Units::from_u8(2),
					},
					offset: None,
				},
			}
		}
	}
}
