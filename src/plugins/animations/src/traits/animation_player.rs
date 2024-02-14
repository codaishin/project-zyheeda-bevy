use super::{RepeatAnimation, ReplayAnimation};
use bevy::animation::AnimationPlayer;

impl RepeatAnimation for AnimationPlayer {
	fn repeat(&mut self, animation: &bevy::prelude::Handle<bevy::prelude::AnimationClip>) {
		self.play(animation.clone()).repeat();
	}
}

impl ReplayAnimation for AnimationPlayer {
	fn replay(&mut self, animation: &bevy::prelude::Handle<bevy::prelude::AnimationClip>) {
		self.play(animation.clone()).replay();
	}
}
