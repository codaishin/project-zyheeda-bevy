use crate::{
	tools::get_recursively::{get_recursively_from, related::Child},
	traits::track::Track,
};
use bevy::prelude::*;

pub trait TrackComponentInChildren<TTracker>
where
	TTracker: Component,
{
	fn track_in_self_and_children<TTarget>(
		mut trackers: Query<(Entity, Mut<TTracker>)>,
		targets_lookup: Query<(), With<TTarget>>,
		trackers_lookup: Query<(), With<TTracker>>,
		children: Query<&Children>,
	) where
		TTarget: Component,
		TTracker: Track<TTarget>,
	{
		if trackers.is_empty() {
			return;
		}

		let children = &|entity| children.get(entity).ok().map(|c| c.iter().map(Child::new));
		let has_target = &|entity: &Entity| targets_lookup.contains(*entity);
		let is_no_tracker = &|Child(entity): &Child| !trackers_lookup.contains(*entity);

		for (entity, mut tracker) in &mut trackers {
			for entity in get_recursively_from(entity, children, is_no_tracker).filter(has_target) {
				tracker.track(entity)
			}
		}
	}
}

impl<TTracker> TrackComponentInChildren<TTracker> for TTracker where TTracker: Component {}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component, Debug, Default, NestedMocks)]
	struct _Tracker {
		mock: Mock_Tracker,
	}

	#[automock]
	impl Track<_Target> for _Tracker {
		fn track(&mut self, entity: Entity) {
			self.mock.track(entity);
		}
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Target;

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn track_target_entity_in_tracker() {
		let mut app = setup();
		let tracker = app.world_mut().spawn(_Target).id();

		app.world_mut()
			.entity_mut(tracker)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_track()
					.times(1)
					.with(eq(tracker))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(_Tracker::track_in_self_and_children::<_Target>);
	}

	#[test]
	fn track_target_entity_from_child_in_tracker() {
		let mut app = setup();
		let tracker = app.world_mut().spawn_empty().id();
		let target = app.world_mut().spawn(_Target).set_parent(tracker).id();

		app.world_mut()
			.entity_mut(tracker)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_track()
					.times(1)
					.with(eq(target))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(_Tracker::track_in_self_and_children::<_Target>);
	}

	#[test]
	fn track_target_entity_of_deep_child_in_tracker() {
		let mut app = setup();
		let tracker = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn_empty().set_parent(tracker).id();
		let deep_child = app.world_mut().spawn(_Target).set_parent(child).id();

		app.world_mut()
			.entity_mut(tracker)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_track()
					.times(1)
					.with(eq(deep_child))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(_Tracker::track_in_self_and_children::<_Target>);
	}

	#[test]
	fn do_nothing_when_target_missing() {
		let mut app = setup();
		let tracker = app.world_mut().spawn_empty().id();
		app.world_mut().spawn_empty().set_parent(tracker);

		app.world_mut()
			.entity_mut(tracker)
			.insert(_Tracker::new().with_mock(|mock: &mut Mock_Tracker| {
				mock.expect_track().never().return_const(());
			}));

		app.world_mut()
			.run_system_once(_Tracker::track_in_self_and_children::<_Target>);
	}

	#[test]
	fn do_not_track_target_entity_of_child_when_track_component_on_child() {
		let mut app = setup();
		let parent = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn(_Target).set_parent(parent).id();

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
					.with(eq(child))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(_Tracker::track_in_self_and_children::<_Target>);
	}
}
