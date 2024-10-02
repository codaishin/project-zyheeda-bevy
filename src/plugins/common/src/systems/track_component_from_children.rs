use crate::{tools::apply_recursively, traits::track::Track};
use bevy::prelude::*;

pub trait TrackComponentInChildren<TComponent>
where
	TComponent: Component,
{
	fn track_in_children_by<TTracker>(
		mut targets: Query<(Entity, Mut<TTracker>)>,
		components_lookup: Query<(), With<TComponent>>,
		targets_lookup: Query<(), With<TTracker>>,
		children: Query<&Children>,
	) where
		TTracker: Component + Track<TComponent>,
	{
		if targets.is_empty() {
			return;
		}

		let get_children = &|entity| children.get(entity).ok().map(|c| c.iter());
		let has_component = &|entity| components_lookup.contains(entity);
		let is_no_tracker = &|entity| !targets_lookup.contains(entity);

		for (entity, mut target) in &mut targets {
			let track = &mut |child| target.track(child);
			apply_recursively(entity, track, get_children, has_component, is_no_tracker);
		}
	}
}

impl<TComponent> TrackComponentInChildren<TComponent> for TComponent where
	TComponent: Component + Clone
{
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component, Debug, Default, NestedMocks)]
	struct _Target {
		mock: Mock_Target,
	}

	#[automock]
	impl Track<_Source> for _Target {
		fn track(&mut self, entity: Entity) {
			self.mock.track(entity);
		}
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Source;

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn track_source_entity_from_child_in_target() {
		let mut app = setup();
		let target = app.world_mut().spawn_empty().id();
		let source = app.world_mut().spawn(_Source).set_parent(target).id();

		app.world_mut()
			.entity_mut(target)
			.insert(_Target::new().with_mock(|mock: &mut Mock_Target| {
				mock.expect_track()
					.times(1)
					.with(eq(source))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(_Source::track_in_children_by::<_Target>);
	}

	#[test]
	fn track_source_entity_of_deep_child_in_target() {
		let mut app = setup();
		let target = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn_empty().set_parent(target).id();
		let deep_child = app.world_mut().spawn(_Source).set_parent(child).id();

		app.world_mut()
			.entity_mut(target)
			.insert(_Target::new().with_mock(|mock: &mut Mock_Target| {
				mock.expect_track()
					.times(1)
					.with(eq(deep_child))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(_Source::track_in_children_by::<_Target>);
	}

	#[test]
	fn do_nothing_when_source_missing() {
		let mut app = setup();
		let target = app.world_mut().spawn_empty().id();
		app.world_mut().spawn_empty().set_parent(target);

		app.world_mut()
			.entity_mut(target)
			.insert(_Target::new().with_mock(|mock: &mut Mock_Target| {
				mock.expect_track().never().return_const(());
			}));

		app.world_mut()
			.run_system_once(_Source::track_in_children_by::<_Target>);
	}

	#[test]
	fn do_not_track_source_entity_of_deep_child_when_target_component_in_between() {
		let mut app = setup();
		let parent = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn_empty().set_parent(parent).id();
		let deep_child = app.world_mut().spawn(_Source).set_parent(child).id();

		app.world_mut()
			.entity_mut(parent)
			.insert(_Target::new().with_mock(|mock: &mut Mock_Target| {
				mock.expect_track().never().return_const(());
			}));
		app.world_mut()
			.entity_mut(child)
			.insert(_Target::new().with_mock(|mock: &mut Mock_Target| {
				mock.expect_track()
					.times(1)
					.with(eq(deep_child))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(_Source::track_in_children_by::<_Target>);
	}
}
