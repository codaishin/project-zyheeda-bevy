use crate::{
	components::{
		movement::{velocity_based::VelocityBased, Movement},
		SetFace,
	},
	events::MoveInputEvent,
};
use bevy::prelude::*;
use common::traits::handles_orientation::Face;

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
			player.try_insert((
				Movement::<VelocityBased>::to(target).remove_on_cleanup::<SetFace>(),
				SetFace(Face::Translation(target)),
			));
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

		let player = app.world().entity(player);

		assert_eq!(
			(
				Some(
					&Movement::<VelocityBased>::to(Vec3::new(1., 2., 3.),)
						.remove_on_cleanup::<SetFace>()
				),
				Some(&SetFace(Face::Translation(Vec3::new(1., 2., 3.)))),
			),
			(
				player.get::<Movement<VelocityBased>>(),
				player.get::<SetFace>(),
			)
		);
	}
}
