use crate::{
	components::{Face, SetFace, SimpleMovement},
	events::MoveInputEvent,
};
use bevy::ecs::{
	entity::Entity,
	event::EventReader,
	query::With,
	system::{Commands, Query},
};
use common::components::Player;

pub(crate) fn move_player_on_event(
	mut commands: Commands,
	mut move_input_events: EventReader<MoveInputEvent>,
	players: Query<Entity, With<Player>>,
) {
	let Ok(player) = players.get_single() else {
		return;
	};
	let Some(mut player) = commands.get_entity(player) else {
		return;
	};

	for event in move_input_events.read() {
		let target = event.0;
		player.try_insert((
			SimpleMovement { target },
			SetFace(Face::Translation(target)),
		));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Face, SetFace, SimpleMovement};
	use bevy::{
		app::{App, Update},
		math::Vec3,
	};
	use common::test_tools::utils::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, move_player_on_event);
		app.add_event::<MoveInputEvent>();

		app
	}

	#[test]
	fn trigger_movement() {
		let mut app = setup();
		let player = app.world.spawn(Player).id();
		app.world.send_event(MoveInputEvent(Vec3::new(1., 2., 3.)));

		app.update();

		let player = app.world.entity(player);

		assert_eq!(
			(
				Some(&SimpleMovement {
					target: Vec3::new(1., 2., 3.)
				}),
				Some(&SetFace(Face::Translation(Vec3::new(1., 2., 3.))))
			),
			(player.get::<SimpleMovement>(), player.get::<SetFace>())
		);
	}
}
