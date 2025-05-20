use bevy::prelude::*;

impl<T> SetPlayerMovement for T {}

pub trait SetPlayerMovement {
	fn set<TEvent, TMovement>(
		mut commands: Commands,
		mut move_input_events: EventReader<TEvent>,
		players: Query<Entity, With<Self>>,
	) where
		Self: Component + Sized,
		TEvent: Event,
		for<'a> TMovement: Component + From<&'a TEvent>,
	{
		let Ok(player) = players.single() else {
			return;
		};
		let Ok(mut player) = commands.get_entity(player) else {
			return;
		};

		for event in move_input_events.read() {
			player.try_insert(TMovement::from(event));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Event)]
	struct _Event(Vec3);

	#[derive(Component, Debug, PartialEq)]
	struct _Movement(Vec3);

	impl From<&_Event> for _Movement {
		fn from(_Event(target): &_Event) -> Self {
			Self(*target)
		}
	}

	#[derive(Component)]
	struct _Player;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Player::set::<_Event, _Movement>);
		app.add_event::<_Event>();

		app
	}

	#[test]
	fn trigger_movement() {
		let mut app = setup();
		let player = app.world_mut().spawn(_Player).id();
		app.world_mut().send_event(_Event(Vec3::new(1., 2., 3.)));

		app.update();

		assert_eq!(
			Some(&_Movement(Vec3::new(1., 2., 3.))),
			app.world().entity(player).get::<_Movement>(),
		);
	}
}
