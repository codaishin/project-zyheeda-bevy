pub(crate) mod animation_player;
pub(crate) mod player_idle;
pub(crate) mod player_movement;

use bevy::{animation::AnimationClip, asset::Handle};

pub(crate) trait RepeatAnimation {
	fn repeat(&mut self, animation: &Handle<AnimationClip>);
}

pub(crate) trait ReplayAnimation {
	fn replay(&mut self, animation: &Handle<AnimationClip>);
}

pub trait HighestPriorityAnimation<TAnimation> {
	fn highest_priority_animation(&self) -> Option<&TAnimation>;
}

pub enum Priority {
	High,
	Middle,
	Low,
}

pub trait InsertAnimation<TAnimation> {
	fn insert(&mut self, animation: TAnimation, priority: Priority);
}

pub trait RemoveAnimation<TAnimation> {
	fn remove(&mut self, animation: TAnimation, priority: Priority);
}
