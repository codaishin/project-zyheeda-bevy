use crate::system_parameters::loadout_activity::LoadoutActivityWriteContext;
use common::traits::{
	handles_loadout::{CurrentTarget, CurrentTargetMut},
	handles_skill_physics::SkillTarget,
};
use std::ops::DerefMut;

impl CurrentTarget for LoadoutActivityWriteContext<'_> {
	fn current_target(&self) -> Option<&SkillTarget> {
		self.target.0.as_ref()
	}
}

impl CurrentTargetMut for LoadoutActivityWriteContext<'_> {
	fn current_target_mut(&mut self) -> &mut Option<SkillTarget> {
		&mut self.target.deref_mut().0
	}
}
