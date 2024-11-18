use crate::traits::{
	AnimationPlayers,
	HighestPriorityAnimation,
	IsPlaying,
	RepeatAnimation,
	ReplayAnimation,
};
use bevy::{ecs::query::QueryData, prelude::*};
use common::{
	resources::Shared,
	traits::{
		animation::{Animation, PlayMode},
		load_asset::Path,
	},
};

impl<TDispatch> PlayAnimationClip for TDispatch {}

pub(crate) trait PlayAnimationClip
where
	Self: Sized,
{
	fn play_animation_clip_via<TAnimationPlayer>(
		mut players: Query<TAnimationPlayer>,
		dispatchers: Query<&Self, Changed<Self>>,
		animations: Res<Shared<Path, AnimationNodeIndex>>,
	) where
		TAnimationPlayer: QueryData,
		for<'a> TAnimationPlayer::Item<'a>: ReplayAnimation<AnimationNodeIndex>
			+ RepeatAnimation<AnimationNodeIndex>
			+ IsPlaying<AnimationNodeIndex>,
		for<'a> Self: Component + AnimationPlayers<'a> + HighestPriorityAnimation<Animation>,
	{
		for dispatcher in &dispatchers {
			for entity in dispatcher.animation_players() {
				let Some(animation) = dispatcher.highest_priority_animation() else {
					continue;
				};
				let Some(index) = animations.get(&animation.path) else {
					continue;
				};
				let Ok(mut player) = players.get_mut(entity) else {
					continue;
				};
				if player.is_playing(*index) {
					continue;
				}

				match animation.play_mode {
					PlayMode::Repeat => player.repeat(*index),
					PlayMode::Replay => player.replay(*index),
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		resources::Shared,
		test_tools::utils::SingleThreadedApp,
		traits::{load_asset::Path, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::{collections::VecDeque, ops::DerefMut};

	#[derive(Component, NestedMocks)]
	struct _AnimationDispatch {
		mock: Mock_AnimationDispatch,
	}

	impl<'a> AnimationPlayers<'a> for _AnimationDispatch {
		type TIter = _Iter;

		fn animation_players(&'a self) -> Self::TIter {
			self.mock.animation_players()
		}
	}

	impl HighestPriorityAnimation<Animation> for _AnimationDispatch {
		fn highest_priority_animation(&self) -> Option<Animation> {
			self.mock.highest_priority_animation()
		}
	}

	mock! {
		_AnimationDispatch {}
		impl<'a> AnimationPlayers<'a> for _AnimationDispatch {
			type TIter = _Iter;

			fn animation_players(&'a self) -> _Iter;
		}
		impl HighestPriorityAnimation<Animation> for _AnimationDispatch {
			fn highest_priority_animation(&self) -> Option<Animation>;
		}
	}

	#[derive(Clone)]
	struct _Iter(VecDeque<Entity>);

	impl<const N: usize> From<[Entity; N]> for _Iter {
		fn from(value: [Entity; N]) -> Self {
			_Iter(VecDeque::from(value))
		}
	}

	impl Iterator for _Iter {
		type Item = Entity;

		fn next(&mut self) -> Option<Self::Item> {
			self.0.pop_front()
		}
	}

	#[derive(Component, NestedMocks)]
	struct _AnimationPlayer {
		mock: Mock_AnimationPlayer,
	}

	impl<'a> ReplayAnimation<AnimationNodeIndex> for Mut<'a, _AnimationPlayer> {
		fn replay(&mut self, index: AnimationNodeIndex) {
			self.mock.replay(index);
		}
	}

	impl<'a> RepeatAnimation<AnimationNodeIndex> for Mut<'a, _AnimationPlayer> {
		fn repeat(&mut self, index: AnimationNodeIndex) {
			self.mock.repeat(index);
		}
	}

	impl<'a> IsPlaying<AnimationNodeIndex> for Mut<'a, _AnimationPlayer> {
		fn is_playing(&self, index: AnimationNodeIndex) -> bool {
			self.mock.is_playing(index)
		}
	}

	mock! {
		_AnimationPlayer {}
		impl ReplayAnimation<AnimationNodeIndex> for _AnimationPlayer {
			fn replay(&mut self, index: AnimationNodeIndex);
		}
		impl RepeatAnimation<AnimationNodeIndex> for _AnimationPlayer {
			fn repeat(&mut self, index: AnimationNodeIndex);
		}
		impl IsPlaying<AnimationNodeIndex> for _AnimationPlayer {
			fn is_playing(&self, index: AnimationNodeIndex) -> bool;
		}
	}

	fn setup<const N: usize>(animations: [(Path, AnimationNodeIndex); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(Shared::new(animations));
		app.add_systems(
			Update,
			_AnimationDispatch::play_animation_clip_via::<Mut<_AnimationPlayer>>,
		);

		app
	}

	#[test]
	fn repeat_animation() {
		let mut app = setup([(Path::from("my/path"), AnimationNodeIndex::new(42))]);
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert))
			.id();
		app.world_mut()
			.spawn(_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_highest_priority_animation()
					.return_const(Animation::new(Path::from("my/path"), PlayMode::Repeat));
			}));

		app.update();

		fn assert(mock: &mut Mock_AnimationPlayer) {
			mock.expect_is_playing().return_const(false);
			mock.expect_replay().never().return_const(());
			mock.expect_repeat()
				.times(1)
				.with(eq(AnimationNodeIndex::new(42)))
				.return_const(());
		}
	}

	#[test]
	fn replay_animation() {
		let mut app = setup([(Path::from("my/path"), AnimationNodeIndex::new(42))]);
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert))
			.id();
		app.world_mut()
			.spawn(_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_highest_priority_animation()
					.return_const(Animation::new(Path::from("my/path"), PlayMode::Replay));
			}));

		app.update();

		fn assert(mock: &mut Mock_AnimationPlayer) {
			mock.expect_is_playing().return_const(false);
			mock.expect_repeat().never().return_const(());
			mock.expect_replay()
				.times(1)
				.with(eq(AnimationNodeIndex::new(42)))
				.return_const(());
		}
	}

	#[test]
	fn do_not_play_when_already_playing() {
		let mut app = setup([(Path::from("my/path"), AnimationNodeIndex::new(42))]);
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert))
			.id();
		app.world_mut()
			.spawn(_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_highest_priority_animation()
					.return_const(Animation::new(Path::from("my/path"), PlayMode::Repeat));
			}));

		app.update();

		fn assert(mock: &mut Mock_AnimationPlayer) {
			mock.expect_is_playing()
				.with(eq(AnimationNodeIndex::new(42)))
				.return_const(true);
			mock.expect_replay().never().return_const(());
			mock.expect_repeat().never().return_const(());
		}
	}

	#[test]
	fn play_animation_only_once() {
		let mut app = setup([(Path::from("my/path"), AnimationNodeIndex::new(42))]);
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert))
			.id();
		app.world_mut()
			.spawn(_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_highest_priority_animation()
					.return_const(Animation::new(Path::from("my/path"), PlayMode::Repeat));
			}));

		app.update();
		app.update();

		fn assert(mock: &mut Mock_AnimationPlayer) {
			mock.expect_is_playing().return_const(false);
			mock.expect_replay().never().return_const(());
			mock.expect_repeat()
				.times(1)
				.with(eq(AnimationNodeIndex::new(42)))
				.return_const(());
		}
	}

	#[test]
	fn play_animation_again_after_dispatcher_mutably_dereferenced() {
		let mut app = setup([(Path::from("my/path"), AnimationNodeIndex::new(42))]);
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert))
			.id();
		let dispatcher = app
			.world_mut()
			.spawn(_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_highest_priority_animation()
					.return_const(Animation::new(Path::from("my/path"), PlayMode::Repeat));
			}))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(dispatcher)
			.get_mut::<_AnimationDispatch>()
			.unwrap()
			.deref_mut();
		app.update();

		fn assert(mock: &mut Mock_AnimationPlayer) {
			mock.expect_is_playing().return_const(false);
			mock.expect_replay().never().return_const(());
			mock.expect_repeat()
				.times(2)
				.with(eq(AnimationNodeIndex::new(42)))
				.return_const(());
		}
	}
}
