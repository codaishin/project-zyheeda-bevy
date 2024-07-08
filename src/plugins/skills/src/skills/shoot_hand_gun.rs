use super::SkillAnimation;
use crate::traits::{AnimationChainIf, GetAnimationSetup};
use animations::animation::{Animation, PlayMode};
use common::{tools::player_animation_path, traits::load_asset::Path};
use std::marker::PhantomData;

pub(crate) struct ShootHandGun<T = ()>(PhantomData<T>);

fn shoot_right() -> Path {
	player_animation_path("Animation4")
}
fn shoot_right_dual() -> Path {
	player_animation_path("Animation6")
}
fn shoot_left() -> Path {
	player_animation_path("Animation5")
}
fn shoot_left_dual() -> Path {
	player_animation_path("Animation7")
}

impl GetAnimationSetup for ShootHandGun {
	fn get_animation() -> SkillAnimation {
		SkillAnimation {
			right: Animation::new(shoot_right(), PlayMode::Repeat),
			left: Animation::new(shoot_left(), PlayMode::Repeat),
		}
	}

	fn get_chains() -> Vec<AnimationChainIf> {
		vec![
			AnimationChainIf {
				last: shoot_right,
				this: shoot_left,
				then: shoot_left_dual,
			},
			AnimationChainIf {
				last: shoot_left,
				this: shoot_right,
				then: shoot_right_dual,
			},
			AnimationChainIf {
				last: shoot_right_dual,
				this: shoot_left,
				then: shoot_left_dual,
			},
			AnimationChainIf {
				last: shoot_left_dual,
				this: shoot_right,
				then: shoot_right_dual,
			},
		]
	}
}
