use crate::{
	AnimationData,
	traits::{AnimationPlayers, HighestPriorityAnimation, RepeatAnimation, ReplayAnimation},
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
	fn play_animation_clip_via<TAnimationPlayer, TAgent>(
		mut players: Query<TAnimationPlayer>,
		dispatchers: Query<&Self, Changed<Self>>,
		mut graphs: ResMut<Assets<AnimationGraph>>,
		animations: Res<Shared<Path, AnimationNodeIndex>>,
		animation_data: Res<AnimationData<TAgent>>,
	) where
		TAnimationPlayer: QueryData,
		TAgent: Component,
		for<'a> TAnimationPlayer::Item<'a>:
			ReplayAnimation<AnimationNodeIndex> + RepeatAnimation<AnimationNodeIndex>,
		for<'a> Self: Component + AnimationPlayers<'a> + HighestPriorityAnimation<Animation>,
	{
		for dispatcher in &dispatchers {
			for entity in dispatcher.animation_players() {
				let Some(playing_animation) = dispatcher.highest_priority_animation() else {
					continue;
				};
				let Some(playing_animation_id) = animations.get(&playing_animation.path) else {
					continue;
				};
				let Ok(mut player) = players.get_mut(entity) else {
					continue;
				};

				match playing_animation.play_mode {
					PlayMode::Repeat => player.repeat(*playing_animation_id),
					PlayMode::Replay => player.replay(*playing_animation_id),
				}

				let Some(graph) = graphs.get_mut(&animation_data.graph) else {
					continue;
				};

				let Some(playing_mask) = animation_data.masks.get(&playing_animation.path) else {
					continue;
				};

				for animation_id in animations.values() {
					let Some(animation_node) = graph.get_mut(*animation_id) else {
						continue;
					};

					match animation_id == playing_animation_id {
						true => unmask(animation_node, *playing_mask),
						false => mask(animation_node, *playing_mask),
					}
				}
			}
		}
	}
}

fn unmask(animation_node: &mut AnimationGraphNode, mask: AnimationMask) {
	animation_node.mask &= !mask;
}

fn mask(animation_node: &mut AnimationGraphNode, mask: AnimationMask) {
	animation_node.mask |= mask;
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		resources::Shared,
		test_tools::utils::{SingleThreadedApp, new_handle},
		traits::{load_asset::Path, nested_mock::NestedMocks},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::{
		collections::{HashMap, VecDeque},
		ops::DerefMut,
	};

	#[derive(Component)]
	struct _Agent;

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

	impl Default for _AnimationPlayer {
		fn default() -> Self {
			let mut mock = Mock_AnimationPlayer::default();
			mock.expect_replay().return_const(());
			mock.expect_repeat().return_const(());
			Self { mock }
		}
	}

	impl ReplayAnimation<AnimationNodeIndex> for Mut<'_, _AnimationPlayer> {
		fn replay(&mut self, index: AnimationNodeIndex) {
			self.mock.replay(index);
		}
	}

	impl RepeatAnimation<AnimationNodeIndex> for Mut<'_, _AnimationPlayer> {
		fn repeat(&mut self, index: AnimationNodeIndex) {
			self.mock.repeat(index);
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
	}

	fn setup<const N: usize>(
		animations: [(Path, AnimationMask); N],
		graph_handle: &Handle<AnimationGraph>,
	) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut graphs = Assets::default();
		let mut graph = AnimationGraph::new();
		let mask_map = HashMap::from(animations.clone().map(|(path, mask)| (path, mask)));
		let index_map =
			animations.map(|(path, _)| (path, graph.add_clip(new_handle(), 1., graph.root)));

		graphs.insert(graph_handle, graph);
		app.insert_resource(Shared::new(index_map));
		app.insert_resource(AnimationData::<_Agent>::new(graph_handle.clone(), mask_map));
		app.insert_resource(graphs);
		app.add_systems(
			Update,
			_AnimationDispatch::play_animation_clip_via::<&mut _AnimationPlayer, _Agent>,
		);

		app
	}

	#[test]
	fn repeat_animation() {
		let handle = new_handle();
		let path = Path::from("my/path");
		let mut app = setup([(path.clone(), AnimationMask::default())], &handle);
		let index = *app
			.world()
			.resource::<Shared<Path, AnimationNodeIndex>>()
			.get(&path)
			.unwrap();
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert_repeat(index)))
			.id();
		app.world_mut()
			.spawn(_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_highest_priority_animation()
					.return_const(Animation::new(path.clone(), PlayMode::Repeat));
			}));

		app.update();

		fn assert_repeat(index: AnimationNodeIndex) -> impl Fn(&mut Mock_AnimationPlayer) {
			move |mock| {
				mock.expect_replay().never().return_const(());
				mock.expect_repeat()
					.times(1)
					.with(eq(index))
					.return_const(());
			}
		}
	}

	#[test]
	fn replay_animation() {
		let handle = new_handle();
		let path = Path::from("my/path");
		let mut app = setup([(path.clone(), AnimationMask::default())], &handle);
		let index = *app
			.world()
			.resource::<Shared<Path, AnimationNodeIndex>>()
			.get(&path)
			.unwrap();
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert_replay(index)))
			.id();
		app.world_mut()
			.spawn(_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_highest_priority_animation()
					.return_const(Animation::new(path.clone(), PlayMode::Replay));
			}));

		app.update();

		fn assert_replay(index: AnimationNodeIndex) -> impl Fn(&mut Mock_AnimationPlayer) {
			move |mock| {
				mock.expect_repeat().never().return_const(());
				mock.expect_replay()
					.times(1)
					.with(eq(index))
					.return_const(());
			}
		}
	}

	#[test]
	fn play_animation_only_once() {
		let path = Path::from("my/path");
		let mut app = setup([(path.clone(), AnimationMask::default())], &new_handle());
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert_repeat_once))
			.id();
		app.world_mut()
			.spawn(_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_highest_priority_animation()
					.return_const(Animation::new(path.clone(), PlayMode::Repeat));
			}));

		app.update();
		app.update();

		fn assert_repeat_once(mock: &mut Mock_AnimationPlayer) {
			mock.expect_replay().never().return_const(());
			mock.expect_repeat().times(1).return_const(());
		}
	}

	#[test]
	fn play_animation_again_after_dispatcher_mutably_dereferenced() {
		let path = Path::from("my/path");
		let mut app = setup([(path.clone(), AnimationMask::default())], &new_handle());
		let animation_player = app
			.world_mut()
			.spawn(_AnimationPlayer::new().with_mock(assert_repeat_twice))
			.id();
		let dispatcher = app
			.world_mut()
			.spawn(_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_highest_priority_animation()
					.return_const(Animation::new(path.clone(), PlayMode::Repeat));
			}))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(dispatcher)
			.get_mut::<_AnimationDispatch>()
			.unwrap()
			.deref_mut();
		app.update();

		fn assert_repeat_twice(mock: &mut Mock_AnimationPlayer) {
			mock.expect_replay().never().return_const(());
			mock.expect_repeat().times(2).return_const(());
		}
	}

	#[test]
	fn unmask_playing_animation() {
		let handle = new_handle();
		let path = Path::from("my/path");
		let mut app = setup([(path.clone(), 1 << 2)], &handle);
		let index = *app
			.world()
			.resource::<Shared<Path, AnimationNodeIndex>>()
			.get(&path)
			.unwrap();
		let mut graphs = app.world_mut().resource_mut::<Assets<AnimationGraph>>();
		let graph = graphs.get_mut(&handle).unwrap();
		graph.get_mut(index).unwrap().mask = 0b1111;
		let animation_player = app.world_mut().spawn(_AnimationPlayer::default()).id();
		app.world_mut()
			.spawn(_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_highest_priority_animation()
					.return_const(Animation::new(path.clone(), PlayMode::Repeat));
			}));

		app.update();

		let graph = app
			.world()
			.resource::<Assets<AnimationGraph>>()
			.get(&handle)
			.unwrap();
		let node = graph.get(index).unwrap();
		let expected = 0b1011;
		assert_eq!(
			expected, node.mask,
			"\n  left bits: {:b}\n right bits: {:b}",
			expected, node.mask
		);
	}

	#[test]
	fn mask_not_playing_animation() {
		let handle = new_handle();
		let playing = Path::from("my/path");
		let not_playing = Path::from("not/plying");
		let mut app = setup(
			[
				(playing.clone(), 1 << 2),
				(not_playing.clone(), AnimationMask::default()),
			],
			&handle,
		);
		let not_playing_index = *app
			.world()
			.resource::<Shared<Path, AnimationNodeIndex>>()
			.get(&not_playing)
			.unwrap();
		let animation_player = app.world_mut().spawn(_AnimationPlayer::default()).id();
		app.world_mut()
			.spawn(_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_animation_players()
					.return_const(_Iter::from([animation_player]));
				mock.expect_highest_priority_animation()
					.return_const(Animation::new(playing.clone(), PlayMode::Repeat));
			}));

		app.update();

		let graph = app
			.world()
			.resource::<Assets<AnimationGraph>>()
			.get(&handle)
			.unwrap();
		let node = graph.get(not_playing_index).unwrap();
		let expected = 0b0100;
		assert_eq!(
			expected, node.mask,
			"\n  left bits: {:b}\n right bits: {:b}",
			expected, node.mask
		);
	}
}
