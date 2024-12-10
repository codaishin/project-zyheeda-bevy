use crate::{
	components::{Face, Movement, SetFace},
	events::MoveInputEvent,
};
use bevy::{
	ecs::{
		entity::Entity,
		event::EventReader,
		query::With,
		system::{Commands, Query},
	},
	prelude::Component,
};

impl<T> ProcessMoveInput for T {}

pub(crate) trait ProcessMoveInput {
	fn process_move_input<TMovementMode>(
		mut commands: Commands,
		mut move_input_events: EventReader<MoveInputEvent>,
		agents: Query<Entity, With<Self>>,
	) where
		Self: Component + Sized,
		TMovementMode: Sync + Send + 'static,
	{
		let Ok(agent) = agents.get_single() else {
			return;
		};
		let Some(mut player) = commands.get_entity(agent) else {
			return;
		};

		for event in move_input_events.read() {
			let target = event.0;
			player.try_insert((
				Movement::<TMovementMode>::to(target).remove_on_cleanup::<SetFace>(),
				SetFace(Face::Translation(target)),
			));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Player;

	#[derive(Debug, PartialEq)]
	struct _Movement;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Player::process_move_input::<_Movement>);
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
					&Movement::<_Movement>::to(Vec3::new(1., 2., 3.),)
						.remove_on_cleanup::<SetFace>()
				),
				Some(&SetFace(Face::Translation(Vec3::new(1., 2., 3.))))
			),
			(player.get::<Movement<_Movement>>(), player.get::<SetFace>())
		);
	}
}
