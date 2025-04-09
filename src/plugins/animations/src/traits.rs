pub(crate) mod animation_player;
pub(crate) mod asset_server;
pub(crate) mod player_idle;
pub(crate) mod tuple_animation_player_transitions;

use bevy::prelude::*;
use common::traits::animation::{AnimationAsset, AnimationPriority};
use std::collections::HashMap;

pub(crate) trait LoadAnimationAssets<TGraph, TIndices> {
	fn load_animation_assets(
		&mut self,
		animations: Vec<AnimationAsset>,
	) -> (TGraph, HashMap<AnimationAsset, TIndices>);
}

pub trait GetActiveAnimations<TAnimation> {
	type TIter<'a>: Iterator<Item = &'a TAnimation>
	where
		Self: 'a,
		TAnimation: 'a;

	fn get_active_animations<TPriority>(&self, priority: TPriority) -> Self::TIter<'_>
	where
		TPriority: Into<AnimationPriority> + 'static;
}

pub trait IsPlaying<TIndex> {
	fn is_playing(&self, index: TIndex) -> bool;
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

pub trait AnimationPlayers
where
	Self::TIter: Iterator<Item = Entity>,
{
	type TIter;
	fn animation_players(&self) -> Self::TIter;
}

pub trait AnimationPlayersWithoutGraph
where
	Self::TIter: Iterator<Item = Entity>,
{
	type TIter;
	fn animation_players_without_graph(&self) -> Self::TIter;
}
