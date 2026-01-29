use crate::traits::send_collision_interaction::PushOngoingInteraction;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use bevy_rapier3d::prelude::CollidingEntities;

impl<T> PushOngoingCollisions for T where
	T: for<'w, 's> SystemParam<Item<'w, 's>: PushOngoingInteraction>
{
}

pub trait PushOngoingCollisions:
	for<'w, 's> SystemParam<Item<'w, 's>: PushOngoingInteraction>
{
	fn push_ongoing_collisions(
		mut ongoing_interactions: StaticSystemParam<Self>,
		collisions: Query<(Entity, &CollidingEntities)>,
	) {
		for (actor, targets) in &collisions {
			for target in targets.iter() {
				ongoing_interactions.push_ongoing_interaction(actor, target);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy_rapier3d::{
		plugin::{PhysicsSet, RapierPhysicsPlugin, TimestepMode},
		prelude::{CollisionEvent, MassModifiedEvent},
		rapier::prelude::CollisionEventFlags,
	};
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

		// Needed for rapier
		app.add_message::<CollisionEvent>();
		app.add_message::<MassModifiedEvent>();
		app.init_resource::<TimestepMode>();

		app.init_resource::<_OngoingCollisions>();
		app.add_systems(
			Update,
			(
				RapierPhysicsPlugin::<()>::get_systems(PhysicsSet::Writeback), // updates `InteractingEntities`
				ResMut::<_OngoingCollisions>::push_ongoing_collisions,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn write_an_interaction_event_for_each_collision() {
		let mut app = setup();
		let actor = app.world_mut().spawn(CollidingEntities::default()).id();
		let target = app.world_mut().spawn_empty().id();
		app.world_mut().write_message(CollisionEvent::Started(
			actor,
			target,
			CollisionEventFlags::empty(),
		));

		app.world_mut()
			.insert_resource(_OngoingCollisions::new().with_mock(move |mock| {
				mock.expect_push_ongoing_interaction()
					.with(eq(actor), eq(target))
					.once()
					.return_const(());
			}));

		app.update();
	}

	#[test]
	fn write_each_frame() {
		let mut app = setup();
		let actor = app.world_mut().spawn(CollidingEntities::default()).id();
		let target = app.world_mut().spawn_empty().id();
		app.world_mut().write_message(CollisionEvent::Started(
			actor,
			target,
			CollisionEventFlags::empty(),
		));

		app.world_mut()
			.insert_resource(_OngoingCollisions::new().with_mock(move |mock| {
				mock.expect_push_ongoing_interaction()
					.with(eq(actor), eq(target))
					.times(3)
					.return_const(());
			}));

		app.update();
		app.update();
		app.update();
	}

	#[test]
	fn stop_writing_when_stopped() {
		let mut app = setup();
		let actor = app.world_mut().spawn(CollidingEntities::default()).id();
		let target = app.world_mut().spawn_empty().id();
		app.world_mut().write_message(CollisionEvent::Started(
			actor,
			target,
			CollisionEventFlags::empty(),
		));

		app.world_mut()
			.insert_resource(_OngoingCollisions::new().with_mock(move |mock| {
				mock.expect_push_ongoing_interaction()
					.with(eq(actor), eq(target))
					.times(2)
					.return_const(());
			}));

		app.update();
		app.update();
		app.world_mut().write_message(CollisionEvent::Stopped(
			actor,
			target,
			CollisionEventFlags::empty(),
		));
		app.update();
	}
}
