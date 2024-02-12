use super::GetBehaviorMeta;
use bevy::utils::default;
use common::{behaviors::meta::BehaviorMeta, skill::SwordStrike, tools::look_from_spawner};

impl GetBehaviorMeta for SwordStrike {
	fn behavior() -> BehaviorMeta {
		BehaviorMeta {
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
		let lazy = SwordStrike::behavior();

		assert_eq!(
			Some(look_from_spawner as usize),
			lazy.transform_fn.map(|f| f as usize)
		);
	}
}
