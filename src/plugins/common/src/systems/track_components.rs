mod builder;

use crate::{
	tools::get_recursively::{get_recursively_from, related::Child},
	traits::{
		clear::Clear,
		read::Read,
		track::{IsTracking, Track, Untrack},
	},
};
use bevy::{
	ecs::{component::Mutable, query::QueryFilter},
	prelude::*,
};
use builder::TrackSystemsBuilder;

impl<TTracker> TrackComponentInSelfAndChildren for TTracker where TTracker: Component {}

pub trait TrackComponentInSelfAndChildren {
	fn track_in_self_and_children<TTarget>() -> TrackSystemsBuilder<Self, TTarget, ()>
	where
		TTarget: Component,
		Self: Component<Mutability = Mutable>
			+ Track<TTarget>
			+ Untrack<TTarget>
			+ IsTracking<TTarget>
			+ Sized,
	{
		TrackSystemsBuilder {
			untrack: |removed_targets: RemovedComponents<TTarget>, trackers: Query<Mut<Self>>| {
				untrack_in_self_and_children(removed_targets, trackers)
			},
			track: track_in_self_and_children::<Self, TTarget, ()>,
		}
	}
}

fn track_in_self_and_children<TTracker, TTarget, TFilter>(
	mut trackers: Query<(Entity, Mut<TTracker>)>,
	targets_lookup: Query<&TTarget, (Added<TTarget>, TFilter)>,
	trackers_lookup: Query<(), With<TTracker>>,
	children: Query<&Children>,
) where
	TTarget: Component,
	TFilter: QueryFilter,
	TTracker: Track<TTarget> + Component<Mutability = Mutable>,
{
	if trackers.is_empty() {
		return;
	}

	let children = &|entity| children.get(entity).ok().map(|c| c.iter().map(Child::new));
	let get_target = &|entity: &Entity| targets_lookup.get(*entity).ok();
	let is_no_tracker = &|Child(entity): &Child| !trackers_lookup.contains(*entity);

	for (entity, mut tracker) in &mut trackers {
		for entity in get_recursively_from(entity, children, is_no_tracker) {
			let Some(target) = get_target(&entity) else {
				continue;
			};
			tracker.track(entity, target);
		}
	}
}

fn untrack_in_self_and_children<TTracker, TTarget, TRemoveEvents>(
	mut removed_targets: TRemoveEvents,
	mut trackers: Query<Mut<TTracker>>,
) where
	TTarget: Component,
	TTracker: IsTracking<TTarget> + Untrack<TTarget> + Component<Mutability = Mutable>,
	for<'a> TRemoveEvents: Clear + Read<'a, TReturn: Iterator<Item = Entity>>,
{
	if trackers.is_empty() {
		removed_targets.clear();
		return;
	}

	for entity in removed_targets.read() {
		for mut tracker in &mut trackers {
			if !tracker.is_tracking(&entity) {
				continue;
			}
			tracker.untrack(&entity);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use macros::{NestedMocks, simple_mock};
	use mockall::{mock, predicate::eq};
	use std::collections::VecDeque;
	use testing::{Mock, NestedMocks, SingleThreadedApp};

	#[derive(Component, Debug, Default, NestedMocks)]
	struct _Tracker {
		mock: Mock_Tracker,
	}

	impl Track<_Target> for _Tracker {
		fn track(&mut self, entity: Entity, target: &_Target) {
			self.mock.track(entity, target);
		}
	}

	impl IsTracking<_Target> for _Tracker {
		fn is_tracking(&self, entity: &Entity) -> bool {
			self.mock.is_tracking(entity)
		}
	}

	impl Untrack<_Target> for _Tracker {
		fn untrack(&mut self, entity: &Entity) {
			self.mock.untrack(entity);
		}
	}

	mock! {
		#[derive(Debug)]
		_Tracker {}
		impl Track<_Target> for _Tracker {
			fn track(&mut self, entity: Entity, target: &_Target);
		}
		impl IsTracking<_Target> for _Tracker {
			fn is_tracking(&self, entity: &Entity) -> bool;
		}
		impl Untrack<_Target> for _Tracker {
			fn untrack(&mut self, entity: &Entity);
		}
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Target;

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Filter;

	#[derive(Clone)]
	struct _Read(VecDeque<Entity>);

	impl<const N: usize> From<[Entity; N]> for _Read {
		fn from(value: [Entity; N]) -> Self {
			Self(VecDeque::from(value))
		}
	}

	impl Iterator for _Read {
		type Item = Entity;

		fn next(&mut self) -> Option<Self::Item> {
			self.0.pop_front()
		}
	}

	simple_mock! {
		_Removed {}
		impl Clear for _Removed {
			fn clear(&mut self);
		}
		impl<'a> Read<'a> for _Removed {
			type TReturn = _Read;

			fn read(&'a mut self) -> _Read;
		}
	}

	impl Clear for In<Mock_Removed> {
		fn clear(&mut self) {
			self.0.clear();
		}
	}

	impl<'a> Read<'a> for In<Mock_Removed> {
		type TReturn = _Read;

		fn read(&'a mut self) -> _Read {
			self.0.read()
		}
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn track_target_entity_in_tracker() -> Result<(), RunSystemError> {
		let mut app = setup();
		let tracker = app.world_mut().spawn(_Target).id();

		app.world_mut()
			.entity_mut(tracker)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_track()
					.times(1)
					.with(eq(tracker), eq(_Target))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(track_in_self_and_children::<_Tracker, _Target, ()>)
	}

	#[test]
	fn track_target_entity_from_child_in_tracker() -> Result<(), RunSystemError> {
		let mut app = setup();
		let tracker = app.world_mut().spawn_empty().id();
		let target = app.world_mut().spawn(_Target).insert(ChildOf(tracker)).id();

		app.world_mut()
			.entity_mut(tracker)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_track()
					.times(1)
					.with(eq(target), eq(_Target))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(track_in_self_and_children::<_Tracker, _Target, ()>)
	}

	#[test]
	fn track_target_entity_of_deep_child_in_tracker() -> Result<(), RunSystemError> {
		let mut app = setup();
		let tracker = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn_empty().insert(ChildOf(tracker)).id();
		let deep_child = app.world_mut().spawn(_Target).insert(ChildOf(child)).id();

		app.world_mut()
			.entity_mut(tracker)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_track()
					.times(1)
					.with(eq(deep_child), eq(_Target))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(track_in_self_and_children::<_Tracker, _Target, ()>)
	}

	#[test]
	fn do_not_track_when_target_not_new() {
		let mut app = setup().single_threaded(Update);
		let tracker = app.world_mut().spawn_empty().id();
		app.world_mut().spawn(_Target).insert(ChildOf(tracker));
		app.world_mut()
			.entity_mut(tracker)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_track().times(1).return_const(());
			}));

		app.add_systems(Update, track_in_self_and_children::<_Tracker, _Target, ()>);
		app.update();
		app.update();
	}

	#[test]
	fn do_not_track_when_target_not_with_filter_component() {
		let mut app = setup().single_threaded(Update);
		let tracker = app.world_mut().spawn_empty().id();
		app.world_mut().spawn(_Target).insert(ChildOf(tracker));
		app.world_mut()
			.entity_mut(tracker)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_track().never().return_const(());
			}));

		app.add_systems(
			Update,
			track_in_self_and_children::<_Tracker, _Target, With<_Filter>>,
		);
		app.update();
	}

	#[test]
	fn do_not_track_target_entity_of_child_when_track_component_on_child()
	-> Result<(), RunSystemError> {
		let mut app = setup();
		let parent = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn(_Target).insert(ChildOf(parent)).id();

		app.world_mut()
			.entity_mut(parent)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_track().never().return_const(());
			}));
		app.world_mut()
			.entity_mut(child)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_track()
					.times(1)
					.with(eq(child), eq(_Target))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(track_in_self_and_children::<_Tracker, _Target, ()>)
	}

	#[test]
	fn untrack_target_entity_in_tracker() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Target).id();
		let removed = Mock_Removed::new_mock(|mock| {
			mock.expect_clear().return_const(());
			mock.expect_read().return_const(_Read::from([entity]));
		});

		app.world_mut()
			.entity_mut(entity)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_is_tracking()
					.times(1)
					.with(eq(entity))
					.return_const(true);
				mock.expect_untrack()
					.times(1)
					.with(eq(entity))
					.return_const(());
			}));

		app.world_mut().run_system_once_with(
			untrack_in_self_and_children::<_Tracker, _Target, In<Mock_Removed>>,
			removed,
		)
	}

	#[test]
	fn do_not_untrack_target_entity_when_not_tracked() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Target).id();
		let removed = Mock_Removed::new_mock(|mock| {
			mock.expect_clear().return_const(());
			mock.expect_read().return_const(_Read::from([entity]));
		});

		app.world_mut()
			.entity_mut(entity)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_is_tracking().return_const(false);
				mock.expect_untrack().times(0).return_const(());
			}));

		app.world_mut().run_system_once_with(
			untrack_in_self_and_children::<_Tracker, _Target, In<Mock_Removed>>,
			removed,
		)
	}

	#[test]
	fn do_not_untrack_target_not_removed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Target).id();
		let removed = Mock_Removed::new_mock(|mock| {
			mock.expect_clear().return_const(());
			mock.expect_read().return_const(_Read::from([]));
		});

		app.world_mut()
			.entity_mut(entity)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_is_tracking().times(0).return_const(false);
				mock.expect_untrack().times(0).return_const(());
			}));

		app.world_mut().run_system_once_with(
			untrack_in_self_and_children::<_Tracker, _Target, In<Mock_Removed>>,
			removed,
		)
	}

	#[test]
	fn untrack_target_of_child_entity_in_tracker() -> Result<(), RunSystemError> {
		let mut app = setup();
		let parent = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn(_Target).insert(ChildOf(parent)).id();
		let removed = Mock_Removed::new_mock(|mock| {
			mock.expect_clear().return_const(());
			mock.expect_read().return_const(_Read::from([child]));
		});

		app.world_mut()
			.entity_mut(parent)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_is_tracking()
					.times(1)
					.with(eq(child))
					.return_const(true);
				mock.expect_untrack()
					.times(1)
					.with(eq(child))
					.return_const(());
			}));

		app.world_mut().run_system_once_with(
			untrack_in_self_and_children::<_Tracker, _Target, In<Mock_Removed>>,
			removed,
		)
	}

	#[test]
	fn clear_removal_events_when_no_trackers_present() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(_Target);
		let removed = Mock_Removed::new_mock(|mock| {
			mock.expect_clear().times(1).return_const(());
			mock.expect_read().return_const(_Read::from([]));
		});

		app.world_mut().run_system_once_with(
			untrack_in_self_and_children::<_Tracker, _Target, In<Mock_Removed>>,
			removed,
		)
	}

	#[test]
	fn do_not_iterate_removal_events_when_no_trackers_present() -> Result<(), RunSystemError> {
		let mut app = setup();
		app.world_mut().spawn(_Target);
		let removed = Mock_Removed::new_mock(|mock| {
			mock.expect_clear().return_const(());
			mock.expect_read().never().return_const(_Read::from([]));
		});

		app.world_mut().run_system_once_with(
			untrack_in_self_and_children::<_Tracker, _Target, In<Mock_Removed>>,
			removed,
		)
	}
}
