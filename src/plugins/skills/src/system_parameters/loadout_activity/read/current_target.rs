use crate::system_parameters::loadout_activity::LoadoutActivityReadContext;
use common::traits::{handles_loadout::CurrentTarget, handles_skill_physics::SkillTarget};

impl CurrentTarget for LoadoutActivityReadContext<'_> {
	fn current_target(&self) -> Option<&SkillTarget> {
		self.target.0.as_ref()
	}
}
