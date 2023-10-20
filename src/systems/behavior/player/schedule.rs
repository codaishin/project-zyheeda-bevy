use crate::{
	components::{BusyExecuting, Player, Queue},
	events::Enqueue,
};
use bevy::prelude::*;

pub fn player_enqueue<TEvent: Copy + Event, TBehavior: From<TEvent> + Send + Sync + 'static>(
	mut commands: Commands,
	mut player: Query<(Entity, &mut Queue<TBehavior>), With<Player>>,
	mut event_reader: EventReader<TEvent>,
	mut enqueue_event_reader: EventReader<Enqueue<TEvent>>,
) {
	for (player, mut queue) in player.iter_mut() {
		for event in enqueue_event_reader.iter() {
			queue.0.push_back(event.0.into());
		}

		for event in event_reader.iter() {
			queue.0.clear();
			queue.0.push_back((*event).into());
			commands.entity(player).remove::<BusyExecuting<TBehavior>>();
		}
	}
}

#[cfg(test)]
mod tests {
	use std::collections::VecDeque;

	use crate::components::BusyExecuting;

	use super::*;
	use bevy::prelude::App;

	#[derive(Event, Clone, Copy)]
	struct Event {
		pub target: Vec3,
	}

	#[derive(PartialEq, Debug)]
	enum Behavior {
		MoveTo(Vec3),
		Jump,
	}

	impl From<Event> for Behavior {
		fn from(event: Event) -> Self {
			Self::MoveTo(event.target)
		}
	}

	fn setup_app() -> App {
		let mut app = App::new();
		app.add_event::<Event>();
		app.add_event::<Enqueue<Event>>();

		app
	}

	#[test]
	fn set_movement() {
		let mut app = setup_app();
		let player = Player { ..default() };
		let queue = Queue::<Behavior>([Behavior::Jump].into());
		let event = Event {
			target: Vec3::new(1., 2., 3.),
		};
		let busy: BusyExecuting<Behavior> = default();

		let player = app.world.spawn((player, queue, busy)).id();

		app.add_systems(Update, player_enqueue::<Event, Behavior>);
		app.world.resource_mut::<Events<Event>>().send(event);
		app.update();

		let player = app.world.entity(player);
		let queue = player.get::<Queue<Behavior>>().unwrap();
		let is_busy = player.contains::<BusyExecuting<Behavior>>();

		assert_eq!(
			(
				false,
				&VecDeque::from([Behavior::MoveTo(Vec3::new(1., 2., 3.))])
			),
			(is_busy, &queue.0)
		)
	}

	#[test]
	fn add_movement() {
		let mut app = setup_app();
		let player = Player { ..default() };
		let queue = Queue::<Behavior>([Behavior::Jump].into());
		let event = Enqueue(Event {
			target: Vec3::new(1., 2., 3.),
		});
		let busy: BusyExecuting<Behavior> = default();

		let player = app.world.spawn((player, queue, busy)).id();

		app.add_systems(Update, player_enqueue::<Event, Behavior>);
		app.world
			.resource_mut::<Events<Enqueue<Event>>>()
			.send(event);
		app.update();

		let player = app.world.entity(player);
		let queue = player.get::<Queue<Behavior>>().unwrap();
		let is_busy = player.contains::<BusyExecuting<Behavior>>();

		assert_eq!(
			(
				true,
				&VecDeque::from([Behavior::Jump, Behavior::MoveTo(Vec3::new(1., 2., 3.))])
			),
			(is_busy, &queue.0)
		)
	}
}
