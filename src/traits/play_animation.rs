pub mod once;
pub mod repeat;

use bevy::{
	animation::{AnimationClip, AnimationPlayer},
	asset::Handle,
};

pub trait PlayAnimation {
	fn play(player: &mut AnimationPlayer, animation: &Handle<AnimationClip>);
}
