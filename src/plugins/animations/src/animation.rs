use crate::traits::{AnimationChainUpdate, AnimationId, AnimationPath, AnimationPlayMode};
use bevy::utils::Uuid;
use common::{
	tools::{Last, This},
	traits::load_asset::Path,
};
use core::fmt::Debug;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PlayMode {
	Replay,
	Repeat,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Animation {
	uuid: Uuid,
	path: Path,
	play_mode: PlayMode,
	pub update_fn: Option<fn(This<Animation>, Last<Animation>)>,
}

impl Animation {
	pub fn new_unique(path: Path, play_mode: PlayMode) -> Self {
		Self {
			uuid: Uuid::new_v4(),
			path,
			play_mode,
			update_fn: None,
		}
	}
}

impl AnimationId for Animation {
	fn animation_id(&self) -> Uuid {
		self.uuid
	}
}

impl AnimationPath for Animation {
	fn animation_path(&self) -> Path {
		self.path.clone()
	}
}

impl AnimationPlayMode for Animation {
	fn animation_play_mode(&self) -> PlayMode {
		self.play_mode
	}
}

impl AnimationChainUpdate for Animation {
	fn chain_update(&mut self, last: &Animation) {
		let Some(update) = self.update_fn else {
			return;
		};

		update(This(self), Last(last))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use mockall::mock;
	use std::ops::Deref;

	#[test]
	fn new() {
		let path = Path::from("a/path");
		let animation = Animation::new_unique(path.clone(), PlayMode::Repeat);

		assert_eq!(
			(path, PlayMode::Repeat),
			(animation.path, animation.play_mode)
		)
	}

	#[test]
	fn two_animations_with_same_values_are_not_equal() {
		let a = Animation::new_unique(Path::from("a/path"), PlayMode::Repeat);
		let b = Animation::new_unique(Path::from("a/path"), PlayMode::Repeat);

		assert_ne!(a, b);
	}

	#[test]
	fn animation_is_equal_to_itself() {
		let animation = Animation::new_unique(Path::from("a/path"), PlayMode::Repeat);

		assert_eq!(animation, animation);
	}

	#[test]
	fn animation_id() {
		let animation = Animation::new_unique(Path::from(""), PlayMode::Repeat);

		assert_eq!(animation.uuid, animation.animation_id());
	}

	#[test]
	fn animation_path() {
		let animation = Animation::new_unique(Path::from("my/path"), PlayMode::Repeat);

		assert_eq!(Path::from("my/path"), animation.animation_path());
	}

	#[test]
	fn animation_play_mode() {
		let animation = Animation::new_unique(Path::from(""), PlayMode::Repeat);

		assert_eq!(PlayMode::Repeat, animation.animation_play_mode());
	}

	trait _CombinationFn {
		fn combination_fn(a: This<Animation>, b: Last<Animation>);
	}

	mock! {
		_CombinationFn {}
		impl _CombinationFn for _CombinationFn {
			fn combination_fn<'a>(a: This<'a, Animation>, b: Last<'a, Animation>);
		}
	}

	#[test]
	fn chain_update_call_arguments() {
		let mut a = Animation::new_unique(Path::from(""), PlayMode::Repeat);
		a.update_fn = Some(Mock_CombinationFn::combination_fn);
		let b = Animation::new_unique(Path::from(""), PlayMode::Repeat);

		let clone_a = a.clone();
		let clone_b = b.clone();
		let func = Mock_CombinationFn::combination_fn_context();
		func.expect()
			.withf(move |a, b| a.deref() == &clone_a && b.deref() == &clone_b)
			.return_const(());

		a.chain_update(&b);
	}
}
