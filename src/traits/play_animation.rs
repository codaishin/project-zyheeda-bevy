pub mod animation_player;

use bevy::{animation::AnimationClip, asset::Handle};

pub trait RepeatAnimation {
	fn repeat(&mut self, animation: &Handle<AnimationClip>);
}

pub trait ReplayAnimation {
	fn replay(&mut self, animation: &Handle<AnimationClip>);
}
