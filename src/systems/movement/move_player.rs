use crate::{
	components::Player,
	traits::{
		get_target::GetTarget,
		movement::{Movement, Seconds},
	},
};
use bevy::prelude::*;

pub fn move_player<
	TWorldPositionEvent: GetTarget + Event,
	TMovementComponent: Movement + Component,
>(
	time: Res<Time>,
	mut event_reader: EventReader<TWorldPositionEvent>,
	mut query: Query<(&mut Player, &TMovementComponent, &mut Transform)>,
) {
	for (mut player, movement, mut transform) in query.iter_mut() {
		player.move_target = event_reader
			.iter()
			.fold(player.move_target, |_, e| e.get_target());
		execute_move(
			&mut transform,
			movement,
			player.move_target,
			time.delta_seconds(),
		);
	}
}

#[inline]
fn execute_move(
	transform: &mut Transform,
	movement: &dyn Movement,
	target: Option<Vec3>,
	delta_time: Seconds,
) {
	let Some(target) = target else {
		return;
	};
	if target == transform.translation {
		return;
	}

	movement.move_towards(transform, target, delta_time);
}

#[cfg(test)]
mod move_player_tests {
	use super::move_player;
	use crate::{
		components::Player,
		traits::{
			get_target::GetTarget,
			movement::{Movement, Seconds},
		},
	};
	use bevy::prelude::*;
	use mockall::{automock, predicate::eq};
	use std::time::Duration;

	#[derive(Event)]
	struct _Event {
		mock: Mock_Event,
	}

	#[derive(Component)]
	struct _Movement {
		mock: Mock_Movement,
	}

	impl _Event {
		pub fn new() -> Self {
			Self {
				mock: Mock_Event::new(),
			}
		}
	}

	impl _Movement {
		pub fn new() -> Self {
			Self {
				mock: Mock_Movement::new(),
			}
		}
	}

	#[automock]
	impl GetTarget for _Event {
		fn get_target(&self) -> Option<Vec3> {
			self.mock.get_target()
		}
	}

	#[automock]
	impl Movement for _Movement {
		fn move_towards(&self, agent: &mut Transform, target: Vec3, delta_time: Seconds) {
			self.mock.move_towards(agent, target, delta_time);
		}
	}

	fn setup_app() -> App {
		let mut app = App::new();
		let mut time = Time::default();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(Update, move_player::<_Event, _Movement>);
		app.add_event::<_Event>();

		app
	}

	#[test]
	fn move_player_once() {
		let mut app = setup_app();
		let mut time = app.world.resource_mut::<Time>();

		let last_update = time.last_update().unwrap();
		let transform = Transform::from_xyz(1., 2., 3.);
		let player = Player { move_target: None };
		let target = Vec3::new(4., 5., 6.);

		let mut event = _Event::new();
		let mut movement = _Movement::new();
		let time_delta = Duration::from_millis(30);

		event.mock.expect_get_target().times(1).return_const(target);

		movement
			.mock
			.expect_move_towards()
			.with(eq(transform), eq(target), eq(time_delta.as_secs_f32()))
			.times(1)
			.return_const(());

		time.update_with_instant(last_update + time_delta);
		app.world.spawn((player, movement, transform));
		app.world.resource_mut::<Events<_Event>>().send(event);

		app.update();
	}

	#[test]
	fn move_player_twice() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let player = Player { move_target: None };
		let target = Vec3::new(4., 5., 6.);

		let mut event = _Event::new();
		let mut movement = _Movement::new();

		event.mock.expect_get_target().times(1).return_const(target);

		movement
			.mock
			.expect_move_towards()
			.times(2)
			.return_const(());

		app.world.spawn((player, movement, transform));
		app.world.resource_mut::<Events<_Event>>().send(event);

		app.update();
		app.update();
	}

	#[test]
	fn do_not_move_if_already_on_target() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let player = Player { move_target: None };
		let target = Vec3::new(1., 2., 3.);

		let mut event = _Event::new();
		let mut movement = _Movement::new();

		event.mock.expect_get_target().times(1).return_const(target);

		movement
			.mock
			.expect_move_towards()
			.times(0)
			.return_const(());

		app.world.spawn((player, movement, transform));
		app.world.resource_mut::<Events<_Event>>().send(event);

		app.update();
	}
}
