use super::PlayAnimation;
use crate::tools::Once;
use bevy::{
	animation::{AnimationClip, AnimationPlayer},
	asset::Handle,
};

impl PlayAnimation for Once {
	fn play(player: &mut AnimationPlayer, animation: &Handle<AnimationClip>) {
		player.play(animation.clone()).replay();
	}
}
