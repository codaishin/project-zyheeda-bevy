use crate::traits::track::Track;
use bevy::prelude::*;

pub trait TrackComponentInChildren<TComponent>
where
	TComponent: Component,
{
	fn track_in_children_by<TTracker>(
		mut targets: Query<(Entity, Mut<TTracker>)>,
		sources: Query<(), With<TComponent>>,
		children: Query<&Children>,
	) where
		TTracker: Component + Track<TComponent>,
	{
		for (entity, target) in &mut targets {
			track_components(entity, target, &children, &sources);
		}
	}
}

fn track_components<TComponent, TTracker>(
	entity: Entity,
	mut target: Mut<TTracker>,
	children: &Query<&Children>,
	components: &Query<(), With<TComponent>>,
) where
	TComponent: Component,
	TTracker: Track<TComponent>,
{
	for child in children.iter_descendants(entity) {
		if !components.contains(child) {
			continue;
		};
		target.track(child);
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
	fn track_source_entity_from_deep_child_in_target() {
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
}
