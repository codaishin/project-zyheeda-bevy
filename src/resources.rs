use bevy::prelude::*;

#[derive(Resource)]
pub struct PlayerAnimations {
	pub idle: Handle<AnimationClip>,
	pub walk: Handle<AnimationClip>,
	pub run: Handle<AnimationClip>,
}
