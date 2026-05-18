use crate::traits::PlayAnimation;

use super::{IsPlaying, RepeatAnimation, ReplayAnimation};
use bevy::prelude::*;
use std::time::Duration;

type AnimationPlayerComponents<'a> = (Mut<'a, AnimationPlayer>, Mut<'a, AnimationTransitions>);

impl IsPlaying<AnimationNodeIndex> for AnimationPlayerComponents<'_> {
	fn is_playing(&self, index: AnimationNodeIndex) -> bool {
		let (player, _) = self;
		player.is_playing_animation(index)
	}
}

impl PlayAnimation<AnimationNodeIndex> for AnimationPlayerComponents<'_> {
	fn play(&mut self, index: AnimationNodeIndex) {
		let (player, transitions) = self;
		transitions.play(player, index, Duration::from_millis(100));
	}
}

impl ReplayAnimation<AnimationNodeIndex> for AnimationPlayerComponents<'_> {
	fn replay(&mut self, index: AnimationNodeIndex) {
		let (player, transitions) = self;
		transitions
			.play(player, index, Duration::from_millis(100))
			.replay();
	}
}

impl RepeatAnimation<AnimationNodeIndex> for AnimationPlayerComponents<'_> {
	fn repeat(&mut self, index: AnimationNodeIndex) {
		let (player, transitions) = self;
		transitions
			.play(player, index, Duration::from_millis(100))
			.repeat();
	}
}
