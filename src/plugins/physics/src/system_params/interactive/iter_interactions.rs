use crate::system_params::interactive::InteractiveContext;
use bevy::prelude::*;
use common::traits::handles_physics::IterInteractions;
use std::{collections::hash_set::Iter, iter::Copied};

impl IterInteractions for InteractiveContext<'_> {
	type TIter<'a>
		= Copied<Iter<'a, Entity>>
	where
		Self: 'a;

	fn iter_interactions(&self) -> Self::TIter<'_> {
		self.interactions.iter().copied()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{collider::ChildColliderOf, collision_domains::Interactive},
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
	use common::traits::{accessors::get::GetContext, handles_physics::Interactions};
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
	fn return_overlapping_entities() -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = app
			.world_mut()
			.spawn((
				RigidBody::Dynamic,
				Interactive,
				Transform::from_xyz(-0.1, 0., 0.),
				CollidingEntities::default(),
				Collider::ball(1.),
				ActiveEvents::COLLISION_EVENTS,
			))
			.id();
		let b = app
			.world_mut()
			.spawn((
				RigidBody::Dynamic,
				Interactive,
				Transform::from_xyz(0.1, 0., 0.),
				Collider::ball(1.),
				ActiveEvents::COLLISION_EVENTS,
			))
			.id();
		app.update();
		app.update();

		let interactions = app
			.world_mut()
			.run_system_once(move |i: InteractiveParam| {
				InteractiveParam::get_context(&i, Interactions { entity: a })
					.iter_interactions()
					.collect::<Vec<_>>()
			})?;

		assert_eq!(vec![b], interactions);
		Ok(())
	}

	#[test]
	fn return_overlapping_root_entities() -> Result<(), RunSystemError> {
		let mut app = setup();
		let a = app.world_mut().spawn(RigidBody::Dynamic).id();
		let b = app.world_mut().spawn(RigidBody::Dynamic).id();
		app.world_mut().spawn((
			ChildColliderOf(a),
			Interactive,
			RigidBody::Dynamic,
			Transform::from_xyz(-0.1, 0., 0.),
			CollidingEntities::default(),
			Collider::ball(1.),
			ActiveEvents::COLLISION_EVENTS,
		));
		app.world_mut().spawn((
			ChildColliderOf(b),
			Interactive,
			RigidBody::Dynamic,
			Transform::from_xyz(0.1, 0., 0.),
			Collider::ball(1.),
			ActiveEvents::COLLISION_EVENTS,
		));
		app.update();
		app.update();

		let interactions = app
			.world_mut()
			.run_system_once(move |i: InteractiveParam| {
				InteractiveParam::get_context(&i, Interactions { entity: a })
					.iter_interactions()
					.collect::<Vec<_>>()
			})?;

		assert_eq!(vec![b], interactions);
		Ok(())
	}
}
