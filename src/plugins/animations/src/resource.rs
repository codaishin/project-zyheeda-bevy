use bevy::{animation::AnimationClip, asset::Handle, ecs::system::Resource};
use std::collections::HashMap;

#[derive(Resource)]
pub struct Animations<T>(pub HashMap<T, Handle<AnimationClip>>);
