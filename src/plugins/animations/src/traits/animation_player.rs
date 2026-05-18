use super::{IsPlaying, RepeatAnimation, ReplayAnimation, StopAnimation};
use crate::traits::PlayAnimation;
use bevy::prelude::*;

impl IsPlaying<AnimationNodeIndex> for Mut<'_, AnimationPlayer> {
	fn is_playing(&self, index: AnimationNodeIndex) -> bool {
		self.is_playing_animation(index)
	}
}

impl PlayAnimation<AnimationNodeIndex> for Mut<'_, AnimationPlayer> {
	fn play(&mut self, index: AnimationNodeIndex) {
		AnimationPlayer::play(self, index);
	}
}

impl ReplayAnimation<AnimationNodeIndex> for Mut<'_, AnimationPlayer> {
	fn replay(&mut self, index: AnimationNodeIndex) {
		AnimationPlayer::play(self, index).replay();
	}
}

impl RepeatAnimation<AnimationNodeIndex> for Mut<'_, AnimationPlayer> {
	fn repeat(&mut self, index: AnimationNodeIndex) {
		AnimationPlayer::play(self, index).repeat();
	}
}

impl StopAnimation<AnimationNodeIndex> for Mut<'_, AnimationPlayer> {
	fn stop_animation(&mut self, index: AnimationNodeIndex) {
		self.stop(index);
	}
}
