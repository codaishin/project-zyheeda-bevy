use bevy::prelude::{EventWriter, ResMut, Resource};

use crate::{events::InteractionEvent, traits::Flush};

pub(crate) fn send_flushed_interactions<TTracker>(
	mut tracker: ResMut<TTracker>,
	mut events: EventWriter<InteractionEvent>,
) where
	TTracker: Resource + Flush<TResult = Vec<InteractionEvent>>,
{
	events.send_batch(tracker.flush());
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::Collision;
	use bevy::{
		app::{App, Update},
		prelude::{Entity, Events},
	};
	use common::{
		components::ColliderRoot,
		test_tools::utils::SingleThreadedApp,
		traits::nested_mock::NestedMock,
	};
	use macros::NestedMock;
	use mockall::automock;

	#[derive(Resource, NestedMock)]
	struct _Tracker {
		mock: Mock_Tracker,
	}

	#[automock]
	impl Flush for _Tracker {
		type TResult = Vec<InteractionEvent>;

		fn flush(&mut self) -> Self::TResult {
			self.mock.flush()
		}
	}

	fn setup(tracker: _Tracker) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(tracker);
		app.add_event::<InteractionEvent>();
		app.add_systems(Update, send_flushed_interactions::<_Tracker>);

		app
	}

	#[test]
	fn call_flush() {
		let mut app = setup(_Tracker::new_mock(|mock| {
			mock.expect_flush().times(1).return_const(vec![]);
		}));

		app.update();
	}

	#[test]
	fn send_events() {
		let a = ColliderRoot(Entity::from_raw(42));
		let b = ColliderRoot(Entity::from_raw(46));
		let mut app = setup(_Tracker::new_mock(|mock| {
			mock.expect_flush().return_const(vec![
				InteractionEvent::of(a).collision(Collision::Started(b))
			]);
		}));

		app.update();

		let events = app.world().resource::<Events<InteractionEvent>>();
		let mut reader = events.get_reader();
		let events = reader.read(events);

		assert_eq!(
			vec![&InteractionEvent::of(a).collision(Collision::Started(b)),],
			events.collect::<Vec<_>>()
		)
	}
}
