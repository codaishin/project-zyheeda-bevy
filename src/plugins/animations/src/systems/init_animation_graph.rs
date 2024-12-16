use crate::{traits::AnimationPlayersWithoutTransitions, AnimationData};
use bevy::prelude::*;
use common::traits::try_insert_on::TryInsertOn;

impl<TAgent> InitAnimationGraph for TAgent where TAgent: Component + Sync + Send + Sized + 'static {}

pub(crate) trait InitAnimationGraph
where
	Self: Component + Sync + Send + Sized + 'static,
{
	fn init_animation_graph_and_transitions<TAnimationDispatch>(
		mut commands: Commands,
		graph: Res<AnimationData<Self>>,
		dispatchers: Query<&TAnimationDispatch, (With<Self>, Changed<TAnimationDispatch>)>,
	) where
		for<'a> TAnimationDispatch: AnimationPlayersWithoutTransitions<'a> + Component,
	{
		for dispatch in &dispatchers {
			for entity in dispatch.animation_players_without_transition() {
				commands.try_insert_on(
					entity,
					(
						AnimationGraphHandle(graph.graph.clone()),
						AnimationTransitions::default(),
					),
				);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::{App, Update};
	use common::{test_tools::utils::SingleThreadedApp, traits::nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::mock;
	use std::{collections::VecDeque, ops::DerefMut};
	use uuid::Uuid;

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, NestedMocks)]
	struct _Dispatch {
		mock: Mock_Dispatch,
	}

	impl<'a> AnimationPlayersWithoutTransitions<'a> for _Dispatch {
		type TIter = _Iter;

		fn animation_players_without_transition(&'a self) -> Self::TIter {
			self.mock.animation_players_without_transition()
		}
	}

	mock! {
		_Dispatch {}
		impl<'a> AnimationPlayersWithoutTransitions<'a> for _Dispatch {
			type TIter = _Iter;
			fn animation_players_without_transition(&'a self) -> _Iter;
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

	fn new_handle<T: Asset>() -> Handle<T> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	fn setup(animation_data: AnimationData<_Agent>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			_Agent::init_animation_graph_and_transitions::<_Dispatch>,
		);
		app.insert_resource(animation_data);

		app
	}

	#[test]
	fn add_animation_graph() {
		let animation_graph_handle = new_handle();
		let mut app = setup(AnimationData::new(animation_graph_handle.clone()));
		let animation_player = app.world_mut().spawn(AnimationPlayer::default()).id();
		app.world_mut().spawn((
			_Agent,
			_Dispatch::new().with_mock(|mock| {
				mock.expect_animation_players_without_transition()
					.return_const(_Iter::from([animation_player]));
			}),
		));

		app.update();

		let animation_player = app.world().entity(animation_player);

		assert_eq!(
			Some(&AnimationGraphHandle(animation_graph_handle)),
			animation_player.get::<AnimationGraphHandle>()
		);
	}

	#[test]
	fn add_animation_transitions() {
		let mut app = setup(AnimationData::new(new_handle()));
		let animation_player = app.world_mut().spawn(AnimationPlayer::default()).id();
		app.world_mut().spawn((
			_Agent,
			_Dispatch::new().with_mock(|mock| {
				mock.expect_animation_players_without_transition()
					.return_const(_Iter::from([animation_player]));
			}),
		));

		app.update();

		let animation_player = app.world().entity(animation_player);

		assert!(animation_player.contains::<AnimationTransitions>());
	}

	#[test]
	fn do_not_add_components_when_no_agent() {
		let mut app = setup(AnimationData::new(new_handle()));
		let animation_player = app.world_mut().spawn(AnimationPlayer::default()).id();
		app.world_mut().spawn(_Dispatch::new().with_mock(|mock| {
			mock.expect_animation_players_without_transition()
				.return_const(_Iter::from([animation_player]));
		}));

		app.update();

		let animation_player = app.world().entity(animation_player);

		assert_eq!(
			(false, false),
			(
				animation_player.contains::<AnimationGraphHandle>(),
				animation_player.contains::<AnimationTransitions>()
			)
		);
	}

	#[test]
	fn add_components_only_once() {
		let mut app = setup(AnimationData::new(new_handle()));
		let animation_player = app.world_mut().spawn(AnimationPlayer::default()).id();
		app.world_mut().spawn((
			_Agent,
			_Dispatch::new().with_mock(|mock| {
				mock.expect_animation_players_without_transition()
					.return_const(_Iter::from([animation_player]));
			}),
		));

		app.update();

		let mut entity = app.world_mut().entity_mut(animation_player);
		entity.remove::<AnimationGraphHandle>();
		entity.remove::<AnimationTransitions>();

		app.update();

		let animation_player = app.world().entity(animation_player);

		assert_eq!(
			(false, false),
			(
				animation_player.contains::<AnimationGraphHandle>(),
				animation_player.contains::<AnimationTransitions>()
			)
		);
	}

	#[test]
	fn add_components_again_when_dispatch_mutably_dereferenced() {
		let mut app = setup(AnimationData::new(new_handle()));
		let animation_player = app.world_mut().spawn(AnimationPlayer::default()).id();
		let agent = app
			.world_mut()
			.spawn((
				_Agent,
				_Dispatch::new().with_mock(|mock| {
					mock.expect_animation_players_without_transition()
						.return_const(_Iter::from([animation_player]));
				}),
			))
			.id();

		app.update();

		let mut entity = app.world_mut().entity_mut(animation_player);
		entity.remove::<AnimationGraphHandle>();
		entity.remove::<AnimationTransitions>();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<_Dispatch>()
			.unwrap()
			.deref_mut();

		app.update();

		let animation_player = app.world().entity(animation_player);

		assert_eq!(
			(true, true),
			(
				animation_player.contains::<AnimationGraphHandle>(),
				animation_player.contains::<AnimationTransitions>()
			)
		);
	}

	#[test]
	fn add_none_when_agent_not_tracked_in_animation_dispatch() {
		let mut app = setup(AnimationData::new(new_handle()));
		let animation_player = app.world_mut().spawn(AnimationPlayer::default()).id();
		app.world_mut().spawn((
			_Agent,
			_Dispatch::new().with_mock(|mock| {
				mock.expect_animation_players_without_transition()
					.return_const(_Iter::from([]));
			}),
		));

		app.update();

		let animation_player = app.world().entity(animation_player);

		assert_eq!(
			(false, false),
			(
				animation_player.contains::<AnimationGraphHandle>(),
				animation_player.contains::<AnimationTransitions>()
			)
		);
	}
}
