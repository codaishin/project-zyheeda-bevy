pub(crate) mod animation_player;
pub(crate) mod player_idle;
pub(crate) mod player_movement;

use crate::animation::PlayMode;
use bevy::{animation::AnimationClip, asset::Handle, utils::Uuid};
use common::traits::load_asset::Path;

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

pub(crate) trait AnimationId {
	fn animation_id(&self) -> Uuid;
}

pub(crate) trait AnimationPath {
	fn animation_path(&self) -> Path;
}

pub(crate) trait AnimationPlayMode {
	fn animation_play_mode(&self) -> PlayMode;
}
