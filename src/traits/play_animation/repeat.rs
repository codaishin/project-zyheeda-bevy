use super::PlayAnimation;
use crate::tools::Repeat;
use bevy::{
	animation::{AnimationClip, AnimationPlayer},
	asset::Handle,
};

impl PlayAnimation for Repeat {
	fn play(player: &mut AnimationPlayer, animation: &Handle<AnimationClip>) {
		player.play(animation.clone()).repeat();
	}
}
