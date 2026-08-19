use crate::resources::game_state_roles::GameStateRoles;
use bevy::prelude::*;

impl GameStateRoles {
	pub(crate) fn pause(In(paused): In<bool>, mut time: ResMut<Time<Virtual>>) {
		if paused {
			time.pause();
		} else {
			time.unpause();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::time::TimePlugin;
	use testing::SingleThreadedApp;

	fn setup(paused: bool) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(TimePlugin);
		app.add_systems(Update, (move || paused).pipe(GameStateRoles::pause));

		app
	}

	#[test]
	fn pause() {
		let mut app = setup(true);
		app.world_mut().resource_mut::<Time<Virtual>>().unpause();

		app.update();

		assert!(app.world().resource::<Time<Virtual>>().is_paused());
	}

	#[test]
	fn un_pause() {
		let mut app = setup(false);
		app.world_mut().resource_mut::<Time<Virtual>>().pause();

		app.update();

		assert!(!app.world().resource::<Time<Virtual>>().is_paused());
	}
}
