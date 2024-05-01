pub(crate) mod animation_player;
pub(crate) mod player_idle;

use crate::animation::PlayMode;
use bevy::{animation::AnimationClip, asset::Handle};
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

#[derive(Debug, PartialEq)]
pub enum Priority {
	High,
	Middle,
	Low,
}

pub trait InsertAnimation<TAnimation> {
	fn insert(&mut self, animation: TAnimation, priority: Priority);
}

pub trait MarkObsolete<TAnimation> {
	fn mark_obsolete(&mut self, priority: Priority);
}

pub(crate) trait FlushObsolete {
	fn flush_obsolete(&mut self, priority: Priority);
}

pub(crate) trait AnimationPath {
	fn animation_path(&self) -> &Path;
}

pub(crate) trait AnimationPlayMode {
	fn animation_play_mode(&self) -> PlayMode;
}

pub trait AnimationChainUpdate {
	fn chain_update(&mut self, last: &Self);
}
