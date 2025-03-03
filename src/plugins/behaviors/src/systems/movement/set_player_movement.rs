use crate::{
	components::movement::{Movement, velocity_based::VelocityBased},
	events::MoveInputEvent,
};
use bevy::prelude::*;

impl<T> SetPlayerMovement for T {}

pub trait SetPlayerMovement {
	fn set_movement(
		mut commands: Commands,
		mut move_input_events: EventReader<MoveInputEvent>,
		players: Query<Entity, With<Self>>,
	) where
		Self: Component + Sized,
	{
		let Ok(player) = players.get_single() else {
			return;
		};
		let Some(mut player) = commands.get_entity(player) else {
			return;
		};

		for event in move_input_events.read() {
			let target = event.0;
			player.try_insert(Movement::<VelocityBased>::to(target));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Player;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Player::set_movement);
		app.add_event::<MoveInputEvent>();

		app
	}

	#[test]
	fn trigger_movement() {
		let mut app = setup();
		let player = app.world_mut().spawn(_Player).id();
		app.world_mut()
			.send_event(MoveInputEvent(Vec3::new(1., 2., 3.)));

		app.update();

		assert_eq!(
			Some(&Movement::<VelocityBased>::to(Vec3::new(1., 2., 3.))),
			app.world().entity(player).get::<Movement<VelocityBased>>(),
		);
	}
}
