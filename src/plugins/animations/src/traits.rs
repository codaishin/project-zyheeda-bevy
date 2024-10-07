pub(crate) mod asset_server;
pub(crate) mod player;
pub(crate) mod player_idle;
pub(crate) mod tuple_animation_player_transitions;

use crate::components::animation_dispatch::AnimationDispatch;
use bevy::prelude::*;
use common::{systems::init_associated_component::GetAssociated, traits::load_asset::Path};
use std::collections::HashMap;

pub(crate) trait LoadAnimationAssets<TGraph, TIndex> {
	fn load_animation_assets(&self, paths: &[Path]) -> (TGraph, HashMap<Path, TIndex>);
}

pub trait HighestPriorityAnimation<TAnimation> {
	fn highest_priority_animation(&self) -> Option<TAnimation>;
}

#[derive(Debug, PartialEq)]
pub enum Priority {
	High,
	Middle,
	Low,
}

pub(crate) trait InsertAnimation<TAnimation> {
	fn insert(&mut self, animation: TAnimation, priority: Priority);
}

pub(crate) trait MarkObsolete {
	fn mark_obsolete(&mut self, priority: Priority);
}

pub(crate) trait FlushObsolete {
	fn flush_obsolete(&mut self, priority: Priority);
}

pub(crate) trait IsPlaying<TIndex> {
	fn is_playing(&self, index: TIndex) -> bool;
}

pub(crate) trait ReplayAnimation<TIndex> {
	fn replay(&mut self, index: TIndex);
}

pub(crate) trait RepeatAnimation<TIndex> {
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

pub trait GetAnimationPaths {
	fn animation_paths() -> Vec<Path>;
}

pub trait RegisterAnimations {
	fn register_animations<
		TAgent: Component + GetAnimationPaths + GetAssociated<AnimationDispatch>,
	>(
		&mut self,
	) -> &mut Self;
}

#[derive(Debug, PartialEq)]
pub struct SkillLayer;

#[derive(Debug, PartialEq)]
pub struct MovementLayer;

#[derive(Debug, PartialEq)]
pub struct IdleLayer;

impl From<SkillLayer> for Priority {
	fn from(_: SkillLayer) -> Self {
		Priority::High
	}
}

impl From<MovementLayer> for Priority {
	fn from(_: MovementLayer) -> Self {
		Priority::Middle
	}
}

impl From<IdleLayer> for Priority {
	fn from(_: IdleLayer) -> Self {
		Priority::Low
	}
}

pub trait StartAnimation<TAnimation> {
	fn start_animation<TLayer>(&mut self, layer: TLayer, animation: TAnimation)
	where
		TLayer: 'static,
		Priority: From<TLayer>;
}

impl<T: InsertAnimation<TAnimation>, TAnimation> StartAnimation<TAnimation> for T {
	fn start_animation<TLayer>(&mut self, layer: TLayer, animation: TAnimation)
	where
		TLayer: 'static,
		Priority: From<TLayer>,
	{
		self.insert(animation, Priority::from(layer));
	}
}

pub trait StopAnimation {
	fn stop_animation<TLayer>(&mut self, layer: TLayer)
	where
		TLayer: 'static,
		Priority: From<TLayer>;
}

impl<T: MarkObsolete> StopAnimation for T {
	fn stop_animation<TLayer>(&mut self, layer: TLayer)
	where
		TLayer: 'static,
		Priority: From<TLayer>,
	{
		self.mark_obsolete(Priority::from(layer));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use mockall::{mock, predicate::eq};

	#[derive(Debug, PartialEq)]
	struct _Animation(&'static str);

	mock! {
		_Dispatch {}
		impl InsertAnimation<_Animation> for _Dispatch {
			fn insert(&mut self, animation: _Animation, priority: Priority);
		}
		impl MarkObsolete for _Dispatch {
			fn mark_obsolete(&mut self, priority: Priority);
		}
	}

	#[test]
	fn start_skill_animation() {
		let dispatch = &mut Mock_Dispatch::default();
		dispatch
			.expect_insert()
			.times(1)
			.with(eq(_Animation("my animation")), eq(Priority::High))
			.return_const(());

		dispatch.start_animation(SkillLayer, _Animation("my animation"));
	}

	#[test]
	fn start_movement_animation() {
		let dispatch = &mut Mock_Dispatch::default();
		dispatch
			.expect_insert()
			.times(1)
			.with(eq(_Animation("my animation")), eq(Priority::Middle))
			.return_const(());

		dispatch.start_animation(MovementLayer, _Animation("my animation"));
	}

	#[test]
	fn start_idle_animation() {
		let dispatch = &mut Mock_Dispatch::default();
		dispatch
			.expect_insert()
			.times(1)
			.with(eq(_Animation("my animation")), eq(Priority::Low))
			.return_const(());

		dispatch.start_animation(IdleLayer, _Animation("my animation"));
	}

	#[test]
	fn stop_skill_animation() {
		let dispatch = &mut Mock_Dispatch::default();
		dispatch
			.expect_mark_obsolete()
			.times(1)
			.with(eq(Priority::High))
			.return_const(());

		dispatch.stop_animation(SkillLayer);
	}

	#[test]
	fn stop_movement_animation() {
		let dispatch = &mut Mock_Dispatch::default();
		dispatch
			.expect_mark_obsolete()
			.times(1)
			.with(eq(Priority::Middle))
			.return_const(());

		dispatch.stop_animation(MovementLayer);
	}

	#[test]
	fn stop_idle_animation() {
		let dispatch = &mut Mock_Dispatch::default();
		dispatch
			.expect_mark_obsolete()
			.times(1)
			.with(eq(Priority::Low))
			.return_const(());

		dispatch.stop_animation(IdleLayer);
	}
}
