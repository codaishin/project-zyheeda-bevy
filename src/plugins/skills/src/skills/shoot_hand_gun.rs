use super::SkillAnimation;
use crate::traits::{AnimationChainIf, GetAnimationSetup};
use animations::animation::{Animation, PlayMode};
use common::tools::player_animation_path;
use std::marker::PhantomData;

pub(crate) struct ShootHandGun<T = ()>(PhantomData<T>);

impl GetAnimationSetup for ShootHandGun {
	fn get_animation() -> SkillAnimation {
		SkillAnimation {
			top_hand_left: Animation::new(player_animation_path("Animation6"), PlayMode::Repeat),
			top_hand_right: Animation::new(player_animation_path("Animation7"), PlayMode::Repeat),
			btm_hand_left: Animation::new(player_animation_path("Animation4"), PlayMode::Repeat),
			btm_hand_right: Animation::new(player_animation_path("Animation5"), PlayMode::Repeat),
		}
	}

	fn get_chains() -> Vec<AnimationChainIf> {
		vec![]
	}
}
