use crate::{
	components::Player,
	traits::{get_target::GetTarget, movement::Movement, set_target::SetTarget},
};
use bevy::prelude::*;

pub fn move_player<
	TWorldPositionEvent: GetTarget + Event,
	TMovementComponent: Movement + SetTarget + Component,
>(
	time: Res<Time>,
	mut event_reader: EventReader<TWorldPositionEvent>,
	mut query: Query<(&mut TMovementComponent, &mut Transform, &Player)>,
) {
	let Ok((mut movement, mut transform, ..)) = query.get_single_mut() else {
		return; //FIXME: Handle properly
	};
	for event in event_reader.iter() {
		movement.set_target(event.get_target());
	}
	movement.update(&mut transform, time.delta_seconds());
}

#[cfg(test)]
mod move_player_tests {
	use super::move_player;
	use crate::{
		components::Player,
		traits::{
			get_target::GetTarget,
			movement::{Movement, Seconds},
			set_target::SetTarget,
		},
	};
	use bevy::prelude::*;
	use mockall::{automock, mock, predicate::eq};
	use std::time::Duration;

	#[derive(Event)]
	struct _Event {
		mock: Mock_Event,
	}

	#[derive(Component)]
	struct _Movement {
		mock: Mock_Combined,
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
				mock: Mock_Combined::new(),
			}
		}
	}

	#[automock]
	impl GetTarget for _Event {
		fn get_target(&self) -> Option<Vec3> {
			self.mock.get_target()
		}
	}

	mock!(
		_Combined {}
		impl Movement for _Combined {
			fn update(&self, agent: &mut Transform, delta_time: Seconds) {}
		}
		impl SetTarget for _Combined {
			fn set_target(&mut self, target: Option<Vec3>) {}
		}
	);

	impl Movement for _Movement {
		fn update(&self, agent: &mut Transform, delta_time: Seconds) {
			self.mock.update(agent, delta_time)
		}
	}

	impl SetTarget for _Movement {
		fn set_target(&mut self, target: Option<Vec3>) {
			self.mock.set_target(target)
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
		let player = Player;
		let target = Vec3::new(4., 5., 6.);

		let mut event = _Event::new();
		let mut movement = _Movement::new();
		let time_delta = Duration::from_millis(30);

		event.mock.expect_get_target().times(1).return_const(target);
		movement
			.mock
			.expect_set_target()
			.with(eq(Some(target)))
			.times(1)
			.return_const(());
		movement
			.mock
			.expect_update()
			.with(eq(transform), eq(time_delta.as_secs_f32()))
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
		let player = Player;
		let target = Vec3::new(4., 5., 6.);

		let mut event = _Event::new();
		let mut movement = _Movement::new();

		event.mock.expect_get_target().times(1).return_const(target);
		movement.mock.expect_set_target().times(1).return_const(());
		movement.mock.expect_update().times(2).return_const(());

		app.world.spawn((player, movement, transform));
		app.world.resource_mut::<Events<_Event>>().send(event);

		app.update();
		app.update();
	}
}
