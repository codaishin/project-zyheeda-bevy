use bevy::{animation::AnimationClip, asset::Handle, ecs::system::Resource};
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct AnimationClips<T>(pub HashMap<T, Handle<AnimationClip>>);
