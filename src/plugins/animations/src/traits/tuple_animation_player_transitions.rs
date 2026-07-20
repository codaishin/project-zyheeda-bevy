use crate::{
	components::animation_dispatch::AnimationState,
	traits::{IsPlaying, OldAnimationState, UpdateAnimation},
};
use bevy::prelude::*;
use std::time::Duration;
use zyheeda_core::prelude::*;

type AnimationPlayerComponents<'a> = (Mut<'a, AnimationPlayer>, Mut<'a, AnimationTransitions>);

impl IsPlaying<AnimationNodeIndex> for AnimationPlayerComponents<'_> {
	fn is_playing(&self, index: AnimationNodeIndex) -> bool {
		let (player, _) = self;
		player.is_playing(index)
	}
}

const TRANSITION: Duration = Duration::from_millis(100);

impl UpdateAnimation<AnimationNodeIndex> for AnimationPlayerComponents<'_> {
	fn update_animation(
		&mut self,
		index: AnimationNodeIndex,
		seek: super::SetTo,
	) -> Option<OldAnimationState> {
		let (player, transitions) = self;

		let old = player
			.animation(index)
			.and_then(|active| F32Finite::try_from(active.seek_time()).ok())
			.map(|seek| OldAnimationState(AnimationState { seek }));

		match seek {
			super::SetTo::Play => {
				transitions.play(player, index, TRANSITION);
			}
			super::SetTo::Replay => {
				transitions.play(player, index, TRANSITION).replay();
			}
			super::SetTo::Repeat => {
				transitions.play(player, index, TRANSITION).repeat();
			}
			super::SetTo::Stop => {
				player.update_animation(index, super::SetTo::Stop);
			}
		};

		old
	}
}
