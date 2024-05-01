use bevy::{animation::AnimationClip, asset::Handle, ecs::system::Resource, utils::default};
use std::collections::HashMap;

#[derive(Resource)]
pub struct AnimationClips<T>(pub HashMap<T, Handle<AnimationClip>>);

impl<T> Default for AnimationClips<T> {
	fn default() -> Self {
		Self(default())
	}
}
