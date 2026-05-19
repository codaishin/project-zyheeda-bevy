use super::UpdateAnimation;
use crate::traits::IsPlaying;
use bevy::prelude::*;

impl IsPlaying<AnimationNodeIndex> for Mut<'_, AnimationPlayer> {
	fn is_playing(&self, index: AnimationNodeIndex) -> bool {
		self.is_playing_animation(index)
	}
}

impl UpdateAnimation<AnimationNodeIndex> for Mut<'_, AnimationPlayer> {
	fn update_animation(&mut self, index: AnimationNodeIndex, seek: super::SetTo) {
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
		}
	}
}
