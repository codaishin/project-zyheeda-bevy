use crate::components::CoolDown;
use bevy::{
	ecs::{
		entity::Entity,
		system::{Commands, Query, Res},
	},
	time::Time,
};

pub(crate) fn update_cool_downs<TTime: Default + Send + Sync + 'static>(
	mut commands: Commands,
	mut cool_downs: Query<(Entity, &mut CoolDown)>,
	time: Res<Time<TTime>>,
) {
	let delta = time.delta();

	for (id, mut cool_down) in &mut cool_downs {
		if cool_down.0 <= delta {
			commands.entity(id).remove::<CoolDown>();
		} else {
			cool_down.0 -= delta;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		time::Real,
	};
	use common::test_tools::utils::{SingleThreadedApp, TickTime};
	use std::time::Duration;

	fn setup() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, update_cool_downs::<Real>);
		app.init_resource::<Time<Real>>();

		app
	}

	#[test]
	fn reduce_by_delta() {
		let mut app = setup();
		let cool_down = app.world.spawn(CoolDown(Duration::from_millis(1000))).id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let cool_down = app.world.entity(cool_down);

		assert_eq!(
			Some(&CoolDown(Duration::from_millis(958))),
			cool_down.get::<CoolDown>()
		);
	}

	#[test]
	fn remove_if_remaining_cool_down_is_zero() {
		let mut app = setup();
		let cool_down = app.world.spawn(CoolDown(Duration::from_millis(42))).id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let cool_down = app.world.entity(cool_down);

		assert_eq!(None, cool_down.get::<CoolDown>());
	}

	#[test]
	fn remove_if_remaining_cool_down_is_negative() {
		let mut app = setup();
		let cool_down = app.world.spawn(CoolDown(Duration::from_millis(10))).id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let cool_down = app.world.entity(cool_down);

		assert_eq!(None, cool_down.get::<CoolDown>());
	}
}
