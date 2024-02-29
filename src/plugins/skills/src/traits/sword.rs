use super::GetExecution;
use crate::skill::{SkillExecution, SwordStrike};

impl GetExecution for SwordStrike {
	fn execution() -> SkillExecution {
		SkillExecution::default()
	}
}
