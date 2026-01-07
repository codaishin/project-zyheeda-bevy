use bevy::prelude::*;
use common::{
	tools::Units,
	traits::handles_skill_physics::{SkillCaster, SkillTarget},
};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct GroundTarget {
	pub caster: SkillCaster,
	pub target: SkillTarget,
	pub max_cast_range: Units,
}

impl GroundTarget {
	#[cfg(test)]
	pub(crate) fn with_caster(caster: SkillCaster) -> Self {
		GroundTarget {
			caster,
			target: SkillTarget::default(),
			max_cast_range: Units::from(f32::INFINITY),
		}
	}

	#[cfg(test)]
	pub(crate) fn with_target(mut self, target: impl Into<SkillTarget>) -> Self {
		self.target = target.into();
		self
	}

	#[cfg(test)]
	pub(crate) fn with_max_range(mut self, max_range: Units) -> Self {
		self.max_cast_range = max_range;
		self
	}
}
