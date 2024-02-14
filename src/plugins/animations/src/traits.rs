pub(crate) mod animation_player;
pub(crate) mod player_movement;

use bevy::{animation::AnimationClip, asset::Handle};

pub(crate) trait RepeatAnimation {
	fn repeat(&mut self, animation: &Handle<AnimationClip>);
}

pub(crate) trait ReplayAnimation {
	fn replay(&mut self, animation: &Handle<AnimationClip>);
}
