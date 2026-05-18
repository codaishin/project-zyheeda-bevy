use crate::traits::{IsPlaying, UpdateAnimation};
use bevy::prelude::*;
use std::time::Duration;

type AnimationPlayerComponents<'a> = (Mut<'a, AnimationPlayer>, Mut<'a, AnimationTransitions>);

impl IsPlaying<AnimationNodeIndex> for AnimationPlayerComponents<'_> {
	fn is_playing(&self, index: AnimationNodeIndex) -> bool {
		let (player, _) = self;
		player.is_playing(index)
	}
}

impl UpdateAnimation<AnimationNodeIndex> for AnimationPlayerComponents<'_> {
	fn update_animation(&mut self, index: AnimationNodeIndex, seek: super::SetTo) {
		const TRANSITION: Duration = Duration::from_millis(100);

		let (player, transitions) = self;

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
		}
	}
}
