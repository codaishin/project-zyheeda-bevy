use crate::{
	components::markers::Physical,
	messages::BeamInteraction,
	system_params::update_ongoing_interactions::UpdateOngoingInteractions,
	traits::send_collision_interaction::PushOngoingInteraction,
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};

impl UpdateOngoingInteractions<'_, '_, Physical> {
	pub(crate) fn push_beam_interactions(
		ongoing_interactions: StaticSystemParam<Self>,
		beam_interactions: MessageReader<BeamInteraction>,
	) {
		push_beam_interactions_internal(ongoing_interactions, beam_interactions)
	}
}

fn push_beam_interactions_internal<T>(
	mut ongoing_interactions: StaticSystemParam<T>,
	mut beam_interactions: MessageReader<BeamInteraction>,
) where
	T: for<'w, 's> SystemParam<Item<'w, 's>: PushOngoingInteraction>,
{
	for BeamInteraction { beam, intersects } in beam_interactions.read() {
		ongoing_interactions.push_ongoing_interaction(*beam, *intersects);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::messages::BeamInteraction;
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

		app.add_message::<BeamInteraction>();
		app.init_resource::<_OngoingCollisions>();
		app.add_systems(
			Update,
			push_beam_interactions_internal::<ResMut<_OngoingCollisions>>,
		);

		app
	}

	#[test]
	fn push_interaction_from_beam_to_target() {
		let mut app = setup();
		let actor = app.world_mut().spawn_empty().id();
		let target = app.world_mut().spawn_empty().id();
		app.world_mut().write_message(BeamInteraction {
			beam: actor,
			intersects: target,
		});

		app.world_mut()
			.insert_resource(_OngoingCollisions::new().with_mock(move |mock| {
				mock.expect_push_ongoing_interaction()
					.with(eq(actor), eq(target))
					.once()
					.return_const(());
			}));

		app.update();
	}
}
