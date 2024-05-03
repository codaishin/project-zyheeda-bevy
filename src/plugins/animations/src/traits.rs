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
pub(crate) enum Priority {
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

pub(crate) trait AnimationPath {
	fn animation_path(&self) -> &Path;
}

pub(crate) trait AnimationPlayMode {
	fn animation_play_mode(&self) -> PlayMode;
}

pub trait AnimationChainUpdate {
	fn chain_update(&mut self, last: &Self);
}

pub struct SkillLayer;
pub struct MovementLayer;
pub struct IdleLayer;

pub trait StartAnimation<TLayer, TAnimation> {
	fn start_animation(&mut self, animation: TAnimation);
}

impl<T: InsertAnimation<TAnimation>, TAnimation> StartAnimation<SkillLayer, TAnimation> for T {
	fn start_animation(&mut self, animation: TAnimation) {
		self.insert(animation, Priority::High);
	}
}

impl<T: InsertAnimation<TAnimation>, TAnimation> StartAnimation<MovementLayer, TAnimation> for T {
	fn start_animation(&mut self, animation: TAnimation) {
		self.insert(animation, Priority::Middle);
	}
}

impl<T: InsertAnimation<TAnimation>, TAnimation> StartAnimation<IdleLayer, TAnimation> for T {
	fn start_animation(&mut self, animation: TAnimation) {
		self.insert(animation, Priority::Low);
	}
}

pub trait StopAnimation<TLayer> {
	fn stop_animation(&mut self);
}

impl<T: MarkObsolete> StopAnimation<SkillLayer> for T {
	fn stop_animation(&mut self) {
		self.mark_obsolete(Priority::High);
	}
}

impl<T: MarkObsolete> StopAnimation<MovementLayer> for T {
	fn stop_animation(&mut self) {
		self.mark_obsolete(Priority::Middle);
	}
}

impl<T: MarkObsolete> StopAnimation<IdleLayer> for T {
	fn stop_animation(&mut self) {
		self.mark_obsolete(Priority::Low);
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

		StartAnimation::<SkillLayer, _Animation>::start_animation(
			dispatch,
			_Animation("my animation"),
		);
	}

	#[test]
	fn start_movement_animation() {
		let dispatch = &mut Mock_Dispatch::default();
		dispatch
			.expect_insert()
			.times(1)
			.with(eq(_Animation("my animation")), eq(Priority::Middle))
			.return_const(());

		StartAnimation::<MovementLayer, _Animation>::start_animation(
			dispatch,
			_Animation("my animation"),
		);
	}

	#[test]
	fn start_idle_animation() {
		let dispatch = &mut Mock_Dispatch::default();
		dispatch
			.expect_insert()
			.times(1)
			.with(eq(_Animation("my animation")), eq(Priority::Low))
			.return_const(());

		StartAnimation::<IdleLayer, _Animation>::start_animation(
			dispatch,
			_Animation("my animation"),
		);
	}

	#[test]
	fn stop_skill_animation() {
		let dispatch = &mut Mock_Dispatch::default();
		dispatch
			.expect_mark_obsolete()
			.times(1)
			.with(eq(Priority::High))
			.return_const(());

		StopAnimation::<SkillLayer>::stop_animation(dispatch);
	}

	#[test]
	fn stop_movement_animation() {
		let dispatch = &mut Mock_Dispatch::default();
		dispatch
			.expect_mark_obsolete()
			.times(1)
			.with(eq(Priority::Middle))
			.return_const(());

		StopAnimation::<MovementLayer>::stop_animation(dispatch);
	}

	#[test]
	fn stop_idle_animation() {
		let dispatch = &mut Mock_Dispatch::default();
		dispatch
			.expect_mark_obsolete()
			.times(1)
			.with(eq(Priority::Low))
			.return_const(());

		StopAnimation::<IdleLayer>::stop_animation(dispatch);
	}
}
