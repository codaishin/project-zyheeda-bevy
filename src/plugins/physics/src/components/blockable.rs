use crate::components::{cast_rays::CastRays, skill_transform::SkillTransforms};
use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq, Clone)]
#[require(GlobalTransform, SkillTransforms, CastRays)]
pub struct Blockable {
	pub(crate) object: PhysicalObject,
	pub(crate) ignore: Option<PersistentEntity>,
}

impl Blockable {
	pub(crate) const fn ignoring(mut self, entity: PersistentEntity) -> Self {
		self.ignore = Some(entity);
		self
	}
}

impl From<PhysicalObject> for Blockable {
	fn from(object: PhysicalObject) -> Self {
		Self {
			object,
			ignore: None,
		}
	}
}
