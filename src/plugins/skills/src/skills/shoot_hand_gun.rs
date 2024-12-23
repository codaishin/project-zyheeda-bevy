use super::SkillAnimation;
use crate::{
	slot_key::SlotKey,
	traits::{AnimationChainIf, GetAnimationSetup},
};
use common::components::Side;
use std::marker::PhantomData;

pub(crate) struct ShootHandGun<T = ()>(PhantomData<T>);

impl GetAnimationSetup for ShootHandGun {
	fn get_animation<TPlayer>() -> SkillAnimation
	where
		TPlayer: crate::traits::PlayerAnimations,
	{
		SkillAnimation {
			top_hand_left: TPlayer::animation(SlotKey::TopHand(Side::Left)),
			top_hand_right: TPlayer::animation(SlotKey::TopHand(Side::Left)),
			btm_hand_left: TPlayer::animation(SlotKey::TopHand(Side::Left)),
			btm_hand_right: TPlayer::animation(SlotKey::TopHand(Side::Left)),
		}
	}

	fn get_chains() -> Vec<AnimationChainIf> {
		vec![]
	}
}
