use super::UpdateAnimation;
use crate::{
	components::animation_dispatch::AnimationState,
	traits::{IsPlaying, OldAnimationState},
};
use bevy::prelude::*;
use zyheeda_core::prelude::*;

impl IsPlaying<AnimationNodeIndex> for Mut<'_, AnimationPlayer> {
	fn is_playing(&self, index: AnimationNodeIndex) -> bool {
		self.is_playing_animation(index)
	}
}

impl UpdateAnimation<AnimationNodeIndex> for Mut<'_, AnimationPlayer> {
	fn update_animation(
		&mut self,
		index: AnimationNodeIndex,
		seek: super::SetTo,
	) -> Option<OldAnimationState> {
		let old = self
			.animation(index)
			.and_then(|active| F32Finite::try_from(active.seek_time()).ok())
			.map(|seek| OldAnimationState(AnimationState { seek }));

		match seek {
			super::SetTo::Play => {
				self.play(index);
			}
			super::SetTo::Replay => {
				self.play(index).replay();
			}
			super::SetTo::Repeat => {
				self.play(index).repeat();
			}

			super::SetTo::Stop => {
				self.stop(index);
			}
		};

		old
	}
}
