use super::{RepeatAnimation, ReplayAnimation};
use bevy::{
	animation::{AnimationClip, AnimationPlayer},
	asset::Handle,
	prelude::AnimationGraph,
};

impl RepeatAnimation for AnimationPlayer {
	fn repeat(&mut self, animation: &Handle<AnimationClip>) {
		let (.., index) = AnimationGraph::from_clip(animation.clone());
		self.play(index).repeat();
	}
}

impl ReplayAnimation for AnimationPlayer {
	fn replay(&mut self, animation: &Handle<AnimationClip>) {
		let (.., index) = AnimationGraph::from_clip(animation.clone());
		if self.is_playing_animation(index) {
			return;
		}
		self.play(index).replay();
	}
}
