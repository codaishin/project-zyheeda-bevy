pub(crate) mod animation_graph;
pub(crate) mod animation_player;
pub(crate) mod player_idle;
pub(crate) mod tuple_animation_player_transitions;

use crate::components::animation_dispatch::AnimationState;
use common::traits::handles_animations::{AnimationClips, AnimationKey, AnimationPriority};
use serde::{Deserialize, Serialize};

pub(crate) trait InsertClips<TIndex>: Sized {
	type TBuffer;

	fn with_buffer() -> (Self, Self::TBuffer);

	fn insert_clips(&mut self, data: &Self::TBuffer, animations: AnimationClips) -> TIndex;
}

pub(crate) trait YoungestToOldestActiveAnimations {
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

pub(crate) trait GetAllActiveAnimations {
	type TIter<'a>: Iterator<Item = &'a AnimationKey>
	where
		Self: 'a;

	fn get_all_active_animations(&self) -> Self::TIter<'_>;
}

pub(crate) trait IsPlaying<TIndex> {
	fn is_playing(&self, index: TIndex) -> bool;
}

pub(crate) trait UpdateAnimation<TIndex> {
	fn update_animation(&mut self, index: TIndex, set_to: SetTo) -> Option<OldAnimationState>;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum SetTo {
	Play,
	Replay,
	Repeat,
	Stop,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct OldAnimationState(pub(crate) AnimationState);
