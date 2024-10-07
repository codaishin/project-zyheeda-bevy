use super::{IsPlaying, RepeatAnimation, ReplayAnimation};
use bevy::prelude::*;
use std::time::Duration;

type Player<'a> = (Mut<'a, AnimationPlayer>, Mut<'a, AnimationTransitions>);

impl<'a> IsPlaying<AnimationNodeIndex> for Player<'a> {
	fn is_playing(&self, index: AnimationNodeIndex) -> bool {
		let (player, _) = self;
		player.is_playing_animation(index)
	}
}

impl<'a> ReplayAnimation<AnimationNodeIndex> for Player<'a> {
	fn replay(&mut self, index: AnimationNodeIndex) {
		let (player, transitions) = self;
		transitions.play(player, index, Duration::ZERO).replay();
	}
}

impl<'a> RepeatAnimation<AnimationNodeIndex> for Player<'a> {
	fn repeat(&mut self, index: AnimationNodeIndex) {
		let (player, transitions) = self;
		transitions.play(player, index, Duration::ZERO).repeat();
	}
}
