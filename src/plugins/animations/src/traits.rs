pub(crate) mod animation_graph;
pub(crate) mod animation_player;
pub(crate) mod player_idle;
pub(crate) mod tuple_animation_player_transitions;

use common::traits::handles_animations::{AnimationClips, AnimationKey, AnimationPriority};

pub(crate) trait InsertClips<TIndex>: Sized {
	type TBuffer;

	fn with_buffer() -> (Self, Self::TBuffer);

	fn insert_clips(&mut self, data: &Self::TBuffer, animations: AnimationClips) -> TIndex;
}

pub trait YoungestToOldestActiveAnimations {
	type TIter<'a>: Iterator<Item = &'a AnimationKey>
	where
		Self: 'a;

	fn youngest_to_oldest_active_animations<TPriority>(
		&self,
		priority: TPriority,
	) -> Self::TIter<'_>
	where
		TPriority: Into<AnimationPriority> + 'static;
}

pub trait GetAllActiveAnimations {
	type TIter<'a>: Iterator<Item = &'a AnimationKey>
	where
		Self: 'a;

	fn get_all_active_animations(&self) -> Self::TIter<'_>;
}

pub trait IsPlaying<TIndex> {
	fn is_playing(&self, index: TIndex) -> bool;
}

pub trait PlayAnimation<TIndex> {
	fn play(&mut self, index: TIndex);
}

pub trait ReplayAnimation<TIndex> {
	fn replay(&mut self, index: TIndex);
}

pub trait RepeatAnimation<TIndex> {
	fn repeat(&mut self, index: TIndex);
}

pub trait StopAnimation<TIndex> {
	fn stop_animation(&mut self, index: TIndex);
}
