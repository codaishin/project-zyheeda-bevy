use super::{IsPlaying, RepeatAnimation, ReplayAnimation};
use bevy::prelude::*;

impl IsPlaying<AnimationNodeIndex> for Mut<'_, AnimationPlayer> {
	fn is_playing(&self, index: AnimationNodeIndex) -> bool {
		self.is_playing_animation(index)
	}
}

impl ReplayAnimation<AnimationNodeIndex> for Mut<'_, AnimationPlayer> {
	fn replay(&mut self, index: AnimationNodeIndex) {
		self.play(index).replay();
	}
}

impl RepeatAnimation<AnimationNodeIndex> for Mut<'_, AnimationPlayer> {
	fn repeat(&mut self, index: AnimationNodeIndex) {
		self.play(index).repeat();
	}
}
