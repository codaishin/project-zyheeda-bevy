use crate::traits::send_collision_interaction::SendCollisionInteraction;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use bevy_rapier3d::prelude::CollisionEvent;

impl<T> SendCollisionEventInteractions for T where
	T: for<'w, 's> SystemParam<Item<'w, 's>: SendCollisionInteraction>
{
}

pub trait SendCollisionEventInteractions:
	for<'w, 's> SystemParam<Item<'w, 's>: SendCollisionInteraction>
{
	fn send_collision_event_interactions(
		mut collisions: EventReader<CollisionEvent>,
		mut sender: StaticSystemParam<Self>,
	) {
		for event in collisions.read() {
			match event {
				CollisionEvent::Started(a, b, ..) => sender.start_interaction(*a, *b),
				CollisionEvent::Stopped(a, b, ..) => sender.end_interaction(*a, *b),
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy_rapier3d::rapier::prelude::CollisionEventFlags;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Sender {
		mock: Mock_Sender,
	}

	impl SendCollisionInteraction for ResMut<'_, _Sender> {
		fn start_interaction(&mut self, a: Entity, b: Entity) {
			self.mock.start_interaction(a, b);
		}

		fn end_interaction(&mut self, a: Entity, b: Entity) {
			self.mock.end_interaction(a, b);
		}
	}

	#[automock]
	impl SendCollisionInteraction for _Sender {
		fn start_interaction(&mut self, a: Entity, b: Entity) {
			self.mock.start_interaction(a, b);
		}

		fn end_interaction(&mut self, a: Entity, b: Entity) {
			self.mock.end_interaction(a, b);
		}
	}

	fn setup(mock: _Sender) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(mock);
		app.add_event::<CollisionEvent>();
		app.add_systems(Update, ResMut::<_Sender>::send_collision_event_interactions);

		app
	}

	#[test]
	fn write_an_interaction_event_for_each_collision() {
		let mut app = setup(_Sender::new().with_mock(assert_send_calls));

		app.world_mut().send_event(CollisionEvent::Started(
			Entity::from_raw(42),
			Entity::from_raw(90),
			CollisionEventFlags::empty(),
		));
		app.world_mut().send_event(CollisionEvent::Stopped(
			Entity::from_raw(9),
			Entity::from_raw(55),
			CollisionEventFlags::empty(),
		));

		app.update();

		fn assert_send_calls(mock: &mut Mock_Sender) {
			mock.expect_start_interaction()
				.once()
				.with(eq(Entity::from_raw(42)), eq(Entity::from_raw(90)))
				.return_const(());
			mock.expect_end_interaction()
				.once()
				.with(eq(Entity::from_raw(9)), eq(Entity::from_raw(55)))
				.return_const(());
		}
	}
}
