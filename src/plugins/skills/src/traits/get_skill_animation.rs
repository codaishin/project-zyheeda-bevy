use super::{GetAnimationSetup, GetSkillAnimation};
use crate::skills::SkillAnimation;
use animations::animation::Animation;
use common::tools::{Last, This};

impl<T: GetAnimationSetup> GetSkillAnimation for T {
	fn animation() -> SkillAnimation {
		if T::get_chains().is_empty() {
			return T::get_animation();
		}

		let SkillAnimation {
			mut left,
			mut right,
		} = T::get_animation();

		left.update_fn = Some(apply_chain::<T>);
		right.update_fn = Some(apply_chain::<T>);
		SkillAnimation { left, right }
	}
}

fn apply_chain<T: GetAnimationSetup>(mut this: This<Animation>, last: Last<Animation>) {
	let chains = T::get_chains();
	let chain = chains
		.iter()
		.find(|chain| this.path == (chain.this)() && last.path == (chain.last)());

	let Some(chain) = chain else {
		return;
	};

	this.path = (chain.then)();
}

#[cfg(test)]
mod test {
	use crate::traits::AnimationChainIf;

	use super::*;
	use animations::animation::PlayMode;
	use common::traits::load_asset::Path;
	use mockall::mock;

	macro_rules! mock_setup {
		($ident:ident) => {
			mock! {
				$ident {}
				impl GetAnimationSetup for $ident {
					fn get_animation() -> SkillAnimation;
					fn get_chains() -> Vec<AnimationChainIf>;
				}
			}
		};
	}

	mock_setup!(_MapAnimation);

	#[test]
	fn map_left_and_right_animation() {
		let left = Animation::new(Path::from("left"), PlayMode::Repeat);
		let right = Animation::new(Path::from("right"), PlayMode::Repeat);
		let get_animation = Mock_MapAnimation::get_animation_context();
		let get_chains = Mock_MapAnimation::get_chains_context();

		get_animation.expect().return_const(SkillAnimation {
			left: left.clone(),
			right: right.clone(),
		});
		get_chains.expect().return_const(vec![]);

		assert_eq!(
			SkillAnimation { left, right },
			Mock_MapAnimation::animation()
		)
	}

	mock_setup!(_CallChain);

	#[test]
	fn add_apply_chain_func_when_chains_present() {
		let mut left = Animation::new(Path::from("left"), PlayMode::Repeat);
		let mut right = Animation::new(Path::from("right"), PlayMode::Repeat);
		let get_animation = Mock_CallChain::get_animation_context();
		let get_chains = Mock_CallChain::get_chains_context();

		get_animation.expect().return_const(SkillAnimation {
			left: left.clone(),
			right: right.clone(),
		});
		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from(""),
			this: || Path::from(""),
			then: || Path::from(""),
		}]);

		left.update_fn = Some(apply_chain::<Mock_CallChain>);
		right.update_fn = Some(apply_chain::<Mock_CallChain>);

		assert_eq!(SkillAnimation { left, right }, Mock_CallChain::animation())
	}

	mock_setup!(_ChainCombo);

	#[test]
	fn apply_chain_combo() {
		let get_chains = Mock_ChainCombo::get_chains_context();

		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from("1"),
			this: || Path::from("2"),
			then: || Path::from("3"),
		}]);

		let mut this = Animation::new(Path::from("2"), PlayMode::Repeat);
		let last = Animation::new(Path::from("1"), PlayMode::Repeat);
		apply_chain::<Mock_ChainCombo>(This(&mut this), Last(&last));

		assert_eq!(Path::from("3"), this.path);
	}

	mock_setup!(_ThisMismatch);

	#[test]
	fn do_not_apply_chain_when_this_mismatch() {
		let get_chains = Mock_ThisMismatch::get_chains_context();

		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from("1"),
			this: || Path::from("2"),
			then: || Path::from("3"),
		}]);

		let mut this = Animation::new(Path::from("2 mismatch"), PlayMode::Repeat);
		let last = Animation::new(Path::from("1"), PlayMode::Repeat);
		apply_chain::<Mock_ThisMismatch>(This(&mut this), Last(&last));

		assert_eq!(Path::from("2 mismatch"), this.path);
	}

	mock_setup!(_LastMismatch);

	#[test]
	fn do_not_apply_chain_when_last_mismatch() {
		let get_chains = Mock_LastMismatch::get_chains_context();

		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from("1"),
			this: || Path::from("2"),
			then: || Path::from("3"),
		}]);

		let mut this = Animation::new(Path::from("2"), PlayMode::Repeat);
		let last = Animation::new(Path::from("1 mismatch"), PlayMode::Repeat);
		apply_chain::<Mock_LastMismatch>(This(&mut this), Last(&last));

		assert_eq!(Path::from("2"), this.path);
	}

	mock_setup!(_DifferentChain);

	#[test]
	fn apply_different_chain() {
		let get_chains = Mock_DifferentChain::get_chains_context();

		get_chains.expect().return_const(vec![AnimationChainIf {
			last: || Path::from("d1"),
			this: || Path::from("d2"),
			then: || Path::from("d3"),
		}]);

		let mut this = Animation::new(Path::from("d2"), PlayMode::Repeat);
		let last = Animation::new(Path::from("d1"), PlayMode::Repeat);
		apply_chain::<Mock_DifferentChain>(This(&mut this), Last(&last));

		assert_eq!(Path::from("d3"), this.path);
	}
}
