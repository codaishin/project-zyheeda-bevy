use bevy::prelude::*;
use common::traits::{accessors::get::GetterMut, handles_behaviors::MovementMode};

impl<T> ToggleWalkRun for T {}

pub(crate) trait ToggleWalkRun {
	fn toggle_walk_run(mut agents: Query<&mut Self>, keys: Res<ButtonInput<KeyCode>>)
	where
		Self: Component + Sized + GetterMut<MovementMode>,
	{
		if !keys.just_pressed(KeyCode::NumpadSubtract) {
			return;
		}

		for mut agent in agents.iter_mut() {
			let mode = agent.get_mut();
			toggle(mode);
		}
	}
}

fn toggle(mode: &mut MovementMode) {
	*mode = match mode {
		MovementMode::Slow => MovementMode::Fast,
		MovementMode::Fast => MovementMode::Slow,
	};
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;

	#[derive(Component, Debug, PartialEq)]
	struct _Player(MovementMode);

	impl GetterMut<MovementMode> for _Player {
		fn get_mut(&mut self) -> &mut MovementMode {
			let _Player(movement_mode) = self;

			movement_mode
		}
	}

	fn setup() -> App {
		let mut app = App::new();
		app.init_resource::<ButtonInput<KeyCode>>();

		app
	}

	#[test]
	fn toggle_player_walk_to_run() {
		let mut app = setup();
		let player = app.world_mut().spawn(_Player(MovementMode::Slow)).id();
		app.world_mut()
			.resource_mut::<ButtonInput<KeyCode>>()
			.press(KeyCode::NumpadSubtract);

		app.world_mut().run_system_once(_Player::toggle_walk_run);

		assert_eq!(
			Some(&_Player(MovementMode::Fast)),
			app.world().entity(player).get::<_Player>()
		);
	}

	#[test]
	fn toggle_player_run_to_walk() {
		let mut app = setup();
		let player = app.world_mut().spawn(_Player(MovementMode::Fast)).id();
		app.world_mut()
			.resource_mut::<ButtonInput<KeyCode>>()
			.press(KeyCode::NumpadSubtract);

		app.world_mut().run_system_once(_Player::toggle_walk_run);

		assert_eq!(
			Some(&_Player(MovementMode::Slow)),
			app.world().entity(player).get::<_Player>()
		);
	}

	#[test]
	fn no_toggle_when_no_input() {
		let mut app = setup();
		let player = app.world_mut().spawn(_Player(MovementMode::Slow)).id();

		app.world_mut().run_system_once(_Player::toggle_walk_run);

		assert_eq!(
			Some(&_Player(MovementMode::Slow)),
			app.world().entity(player).get::<_Player>()
		);
	}
}
