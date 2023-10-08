use crate::{
	behavior::MovementMode,
	components::Player,
	traits::{get::GetMut, movement::Movement},
};
use bevy::prelude::*;

pub fn execute<
	TMovementComponent: Movement,
	TState: GetMut<(TMovementComponent, MovementMode)> + Component,
>(
	time: Res<Time>,
	mut query: Query<(&mut TState, &mut Transform, &Player)>,
) {
	let Ok((mut state, mut transform, player)) = query.get_single_mut() else {
		return; //FIXME: Handle properly
	};
	let Some((movement, mode)) = state.get() else {
		return;
	};

	let speed = match mode {
		MovementMode::Walk => player.movement_speed.to_f32(),
		MovementMode::Run => player.run_speed.to_f32(),
	};
	movement.update(&mut transform, time.delta_seconds() * speed);
}

#[cfg(test)]
mod move_player_tests {
	use super::*;
	use crate::{
		components::{Player, UnitsPerSecond},
		traits::movement::{Movement, Units},
	};
	use mockall::{automock, predicate::eq};
	use std::time::Duration;

	struct _Movement;

	#[automock]
	impl Movement for _Movement {
		fn update(&mut self, _agent: &mut Transform, _delta_time: Units) {}
	}

	#[derive(Component)]
	struct _State {
		movement: Option<(Mock_Movement, MovementMode)>,
	}

	impl GetMut<(Mock_Movement, MovementMode)> for _State {
		fn get(&mut self) -> Option<&mut (Mock_Movement, MovementMode)> {
			self.movement.as_mut()
		}
	}

	fn setup_app() -> App {
		let mut app = App::new();
		let mut time = Time::default();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(Update, execute::<Mock_Movement, _State>);

		app
	}

	#[test]
	fn move_player_once() {
		let mut app = setup_app();
		let mut time = app.world.resource_mut::<Time>();

		let last_update = time.last_update().unwrap();
		let transform = Transform::from_xyz(1., 2., 3.);
		let player = Player {
			movement_speed: UnitsPerSecond::new(5.),
			run_speed: UnitsPerSecond::new(10.),
		};
		let time_delta = Duration::from_millis(30);
		let mut movement = Mock_Movement::new();

		movement
			.expect_update()
			.with(eq(transform), eq(time_delta.as_secs_f32() * 5.))
			.times(1)
			.return_const(());

		time.update_with_instant(last_update + time_delta);
		app.world.spawn((
			_State {
				movement: Some((movement, MovementMode::Walk)),
			},
			player,
			transform,
		));

		app.update();
	}

	#[test]
	fn move_player_twice() {
		let mut app = setup_app();
		let transform = Transform::from_xyz(1., 2., 3.);
		let player = Player {
			movement_speed: UnitsPerSecond::new(5.),
			run_speed: UnitsPerSecond::new(10.),
		};
		let mut movement = Mock_Movement::new();

		movement.expect_update().times(2).return_const(());

		app.world.spawn((
			_State {
				movement: Some((movement, MovementMode::Walk)),
			},
			player,
			transform,
		));

		app.update();
		app.update();
	}

	#[test]
	fn move_player_with_run_speed() {
		let mut app = setup_app();
		let mut time = app.world.resource_mut::<Time>();

		let last_update = time.last_update().unwrap();
		let transform = Transform::from_xyz(1., 2., 3.);
		let player = Player {
			movement_speed: UnitsPerSecond::new(5.),
			run_speed: UnitsPerSecond::new(10.),
		};
		let time_delta = Duration::from_millis(30);
		let mut movement = Mock_Movement::new();

		movement
			.expect_update()
			.with(eq(transform), eq(time_delta.as_secs_f32() * 10.))
			.times(1)
			.return_const(());

		time.update_with_instant(last_update + time_delta);
		app.world.spawn((
			_State {
				movement: Some((movement, MovementMode::Run)),
			},
			player,
			transform,
		));

		app.update();
	}
}
