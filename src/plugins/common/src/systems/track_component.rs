use crate::traits::track::Track;
use bevy::prelude::*;

pub trait TrackComponent<TComponent>
where
	TComponent: Component,
{
	fn track_by<TTracker>(mut entities: Query<(Entity, Mut<TTracker>), With<TComponent>>)
	where
		TTracker: Component + Track<TComponent>,
	{
		for (entity, mut target) in &mut entities {
			target.track(entity);
		}
	}
}

impl<TComponent> TrackComponent<TComponent> for TComponent where TComponent: Component + Clone {}

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
	fn track_source_entity_in_target() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Source).id();

		app.world_mut()
			.entity_mut(entity)
			.insert(_Target::new().with_mock(|mock: &mut Mock_Target| {
				mock.expect_track()
					.times(1)
					.with(eq(entity))
					.return_const(());
			}));

		app.world_mut()
			.run_system_once(_Source::track_by::<_Target>);
	}

	#[test]
	fn do_nothing_when_source_missing() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.entity_mut(entity)
			.insert(_Target::new().with_mock(|mock: &mut Mock_Target| {
				mock.expect_track().never().return_const(());
			}));

		app.world_mut()
			.run_system_once(_Source::track_by::<_Target>);
	}
}
