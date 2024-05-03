use super::GetExecution;
use crate::skills::{SkillExecution, SwordStrike};

impl GetExecution for SwordStrike {
	fn execution() -> SkillExecution {
		SkillExecution::default()
	}
}
