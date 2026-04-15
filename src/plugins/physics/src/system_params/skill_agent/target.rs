use crate::system_params::skill_agent::SkillAgentContextMut;
use common::traits::handles_skill_physics::{SkillTarget, Target, TargetMut};

impl Target for SkillAgentContextMut {
	fn target(&self) -> Option<&SkillTarget> {
		todo!()
	}
}

impl TargetMut for SkillAgentContextMut {
	fn target_mut(&mut self) -> &mut Option<SkillTarget> {
		todo!()
	}
}
