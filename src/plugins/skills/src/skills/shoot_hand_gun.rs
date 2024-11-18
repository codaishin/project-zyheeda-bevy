use super::SkillAnimation;
use crate::traits::{AnimationChainIf, GetAnimationSetup};
use common::traits::animation::{Animation, PlayMode};
use player::components::player::Player;
use std::marker::PhantomData;

pub(crate) struct ShootHandGun<T = ()>(PhantomData<T>);

impl GetAnimationSetup for ShootHandGun {
	fn get_animation() -> SkillAnimation {
		SkillAnimation {
			top_hand_left: Animation::new(Player::animation_path("Animation6"), PlayMode::Repeat),
			top_hand_right: Animation::new(Player::animation_path("Animation7"), PlayMode::Repeat),
			btm_hand_left: Animation::new(Player::animation_path("Animation4"), PlayMode::Repeat),
			btm_hand_right: Animation::new(Player::animation_path("Animation5"), PlayMode::Repeat),
		}
	}

	fn get_chains() -> Vec<AnimationChainIf> {
		vec![]
	}
}
