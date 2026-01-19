use crate::{
	systems::ray_cast::map_ray_cast_results_to_interactions::RayInteraction,
	traits::send_collision_interaction::PushOngoingInteraction,
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};

impl<T> PushOngoingBeamCollisions for T where
	T: for<'w, 's> SystemParam<Item<'w, 's>: PushOngoingInteraction>
{
}

pub(crate) trait PushOngoingBeamCollisions:
	for<'w, 's> SystemParam<Item<'w, 's>: PushOngoingInteraction>
{
	fn push_ongoing_beam_collisions(
		In(interactions): In<Vec<RayInteraction>>,
		mut ongoing_interactions: StaticSystemParam<Self>,
	) {
		for RayInteraction { ray, intersecting } in interactions {
			ongoing_interactions.push_ongoing_interaction(ray, intersecting);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _OngoingCollisions {
		mock: Mock_OngoingCollisions,
	}

	impl Default for _OngoingCollisions {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_push_ongoing_interaction().return_const(());
			})
		}
	}

	impl PushOngoingInteraction for ResMut<'_, _OngoingCollisions> {
		fn push_ongoing_interaction(&mut self, a: Entity, b: Entity) {
			self.mock.push_ongoing_interaction(a, b);
		}
	}

	#[automock]
	impl PushOngoingInteraction for _OngoingCollisions {
		fn push_ongoing_interaction(&mut self, a: Entity, b: Entity) {
			self.mock.push_ongoing_interaction(a, b);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_OngoingCollisions>();

		app
	}

	#[test]
	fn push_ray_intersections() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.insert_resource(_OngoingCollisions::new().with_mock(|mock| {
			mock.expect_push_ongoing_interaction()
				.with(eq(Entity::from_raw(11)), eq(Entity::from_raw(42)))
				.once()
				.return_const(());
		}));

		app.world_mut().run_system_once_with(
			ResMut::<_OngoingCollisions>::push_ongoing_beam_collisions,
			vec![RayInteraction {
				ray: Entity::from_raw(11),
				intersecting: Entity::from_raw(42),
			}],
		)?;
		Ok(())
	}
}
