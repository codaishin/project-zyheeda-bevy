pub mod animation_player;
pub mod once;
pub mod repeat;

use bevy::{
	animation::{AnimationClip, AnimationPlayer},
	asset::Handle,
};

pub trait PlayAnimation {
	fn play(player: &mut AnimationPlayer, animation: &Handle<AnimationClip>);
}

pub trait RepeatAnimation {
	fn repeat(&mut self, animation: &Handle<AnimationClip>);
}

pub trait ReplayAnimation {
	fn replay(&mut self, animation: &Handle<AnimationClip>);
}
