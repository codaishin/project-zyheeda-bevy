use crate::components::on_cool_down::OnCoolDown;
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

pub(crate) fn update_cool_downs<TTime: Default + Send + Sync + 'static>(
	mut commands: ZyheedaCommands,
	mut cool_downs: Query<(Entity, &mut OnCoolDown)>,
	time: Res<Time<TTime>>,
) {
	let delta = time.delta();

	for (id, mut cool_down) in &mut cool_downs {
		if cool_down.0 <= delta {
			commands.try_apply_on(&id, |mut e| {
				e.try_remove::<OnCoolDown>();
			});
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
	use std::time::Duration;
	use testing::{SingleThreadedApp, TickTime};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, update_cool_downs::<Real>);
		app.init_resource::<Time<Real>>();

		app
	}

	#[test]
	fn reduce_by_delta() {
		let mut app = setup();
		let cool_down = app
			.world_mut()
			.spawn(OnCoolDown(Duration::from_millis(1000)))
			.id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let cool_down = app.world().entity(cool_down);

		assert_eq!(
			Some(&OnCoolDown(Duration::from_millis(958))),
			cool_down.get::<OnCoolDown>()
		);
	}

	#[test]
	fn remove_if_remaining_cool_down_is_zero() {
		let mut app = setup();
		let cool_down = app
			.world_mut()
			.spawn(OnCoolDown(Duration::from_millis(42)))
			.id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let cool_down = app.world().entity(cool_down);

		assert_eq!(None, cool_down.get::<OnCoolDown>());
	}

	#[test]
	fn remove_if_remaining_cool_down_is_negative() {
		let mut app = setup();
		let cool_down = app
			.world_mut()
			.spawn(OnCoolDown(Duration::from_millis(10)))
			.id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let cool_down = app.world().entity(cool_down);

		assert_eq!(None, cool_down.get::<OnCoolDown>());
	}
}
