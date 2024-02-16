use super::GetExecution;
use crate::{
	skill::{SkillExecution, SwordStrike},
	tools::look_from_spawner,
};
use bevy::utils::default;

impl GetExecution for SwordStrike {
	fn execution() -> SkillExecution {
		SkillExecution {
			transform_fn: Some(look_from_spawner),
			..default()
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn use_proper_transform_fn() {
		let lazy = SwordStrike::execution();

		assert_eq!(
			Some(look_from_spawner as usize),
			lazy.transform_fn.map(|f| f as usize)
		);
	}
}
