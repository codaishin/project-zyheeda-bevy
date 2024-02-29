use super::{RepeatAnimation, ReplayAnimation};
use bevy::{
	animation::{AnimationClip, AnimationPlayer},
	asset::Handle,
};

impl RepeatAnimation for AnimationPlayer {
	fn repeat(&mut self, animation: &Handle<AnimationClip>) {
		self.play(animation.clone()).repeat();
	}
}

impl ReplayAnimation for AnimationPlayer {
	fn replay(&mut self, animation: &Handle<AnimationClip>) {
		if self.animation_clip() == animation {
			return;
		}
		self.play(animation.clone()).replay();
	}
}
