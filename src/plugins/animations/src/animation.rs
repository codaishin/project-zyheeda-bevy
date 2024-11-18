use crate::traits::AnimationChainUpdate;
use common::{
	tools::{Last, This},
	traits::animation::Animation,
};

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
	use common::traits::{animation::PlayMode, load_asset::Path};
	use mockall::mock;
	use std::ops::Deref;

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
