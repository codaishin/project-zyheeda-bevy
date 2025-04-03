pub(crate) mod asset_server;
pub(crate) mod player_idle;
pub(crate) mod tuple_animation_player_transitions;

use bevy::prelude::*;
use common::traits::load_asset::Path;
use std::collections::HashMap;

pub(crate) trait LoadAnimationAssets<TGraph, TIndex> {
	fn load_animation_assets(&self, animations: Vec<Path>) -> (TGraph, HashMap<Path, TIndex>);
}

pub trait HighestPriorityAnimation<TAnimation> {
	fn highest_priority_animation(&self) -> Option<TAnimation>;
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

pub trait AnimationChainUpdate {
	fn chain_update(&mut self, last: &Self);
}

pub trait AnimationPlayers<'a>
where
	Self::TIter: Iterator<Item = Entity>,
{
	type TIter;
	fn animation_players(&'a self) -> Self::TIter;
}

pub trait AnimationPlayersWithoutTransitions<'a>
where
	Self::TIter: Iterator<Item = Entity>,
{
	type TIter;
	fn animation_players_without_transition(&'a self) -> Self::TIter;
}
