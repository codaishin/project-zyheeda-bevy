use crate::traits::AnimationChainUpdate;
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
	pub path: Path,
	pub(crate) play_mode: PlayMode,
	pub update_fn: Option<fn(This<Animation>, Last<Animation>)>,
}

impl Animation {
	pub fn new(path: Path, play_mode: PlayMode) -> Self {
		Self {
			path,
			play_mode,
			update_fn: None,
		}
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
		let animation = Animation::new(path.clone(), PlayMode::Repeat);

		assert_eq!(
			(path, PlayMode::Repeat),
			(animation.path, animation.play_mode)
		)
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
		let mut a = Animation::new(Path::from(""), PlayMode::Repeat);
		a.update_fn = Some(Mock_CombinationFn::combination_fn);
		let b = Animation::new(Path::from(""), PlayMode::Repeat);

		let clone_a = a.clone();
		let clone_b = b.clone();
		let func = Mock_CombinationFn::combination_fn_context();
		func.expect()
			.withf(move |a, b| a.deref() == &clone_a && b.deref() == &clone_b)
			.return_const(());

		a.chain_update(&b);
	}
}
