use super::GetBehaviorMeta;
use crate::{behaviors::meta::BehaviorMeta, markers::Sword, tools::look_from_spawner};
use bevy::utils::default;

impl GetBehaviorMeta for Sword {
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
		let lazy = Sword::behavior();

		assert_eq!(
			Some(look_from_spawner as usize),
			lazy.transform_fn.map(|f| f as usize)
		);
	}
}
