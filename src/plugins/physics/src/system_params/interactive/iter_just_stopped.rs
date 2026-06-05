use crate::system_params::interactive::JustStoppedInteractionsContext;
use bevy::prelude::*;
use common::traits::handles_physics::IterInteractions;
use std::{collections::hash_set::Iter, iter::Copied};

impl IterInteractions for JustStoppedInteractionsContext {
	type TIter<'a>
		= Copied<Iter<'a, Entity>>
	where
		Self: 'a;

	fn iter_interactions(&self) -> Self::TIter<'_> {
		self.just_stopped.iter().copied()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::collision_domains::Interactive,
		resources::root_collisions::RootCollisions,
		system_params::{
			interactive::InteractiveParam,
			update_root_collisions::UpdateRootCollisions,
		},
		systems::interactions::push_ongoing_collisions::PushOngoingCollisions,
		tests::TestCollisionsPlugin,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use bevy_rapier3d::prelude::*;
	use common::traits::{accessors::get::GetContext, handles_physics::InteractionsJustStopped};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(TestCollisionsPlugin);
		app.init_resource::<RootCollisions<Interactive>>();
		app.add_systems(
			Update,
			(
				RootCollisions::<Interactive>::clear,
				UpdateRootCollisions::<Interactive>::push_ongoing_collisions,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn return_entities_that_stopped_interacting() -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = app
			.world_mut()
			.spawn((
				RigidBody::Fixed,
				Interactive,
				Transform::from_xyz(-0.1, 0., 0.),
				CollidingEntities::default(),
				Collider::ball(1.),
				ActiveCollisionTypes::STATIC_STATIC,
				ActiveEvents::COLLISION_EVENTS,
			))
			.id();
		let b = app
			.world_mut()
			.spawn((
				RigidBody::Fixed,
				Interactive,
				Transform::from_xyz(0.1, 0., 0.),
				Collider::ball(1.),
				ActiveCollisionTypes::STATIC_STATIC,
				ActiveEvents::COLLISION_EVENTS,
			))
			.id();
		app.update();
		app.update();
		app.world_mut().entity_mut(b).despawn();
		app.update();

		let interactions = app
			.world_mut()
			.run_system_once(move |i: InteractiveParam| {
				InteractiveParam::get_context(&i, InteractionsJustStopped { entity: a })
					.iter_interactions()
					.collect::<Vec<_>>()
			})?;

		assert_eq!(vec![b], interactions);
		Ok(())
	}
}
