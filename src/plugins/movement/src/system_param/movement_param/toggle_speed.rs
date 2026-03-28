use crate::{components::config::SpeedIndex, system_param::movement_param::MovementContextMut};
use bevy::prelude::*;
use common::traits::handles_movement::ToggleSpeed;

impl<TMotion> ToggleSpeed for MovementContextMut<'_, TMotion>
where
	TMotion: Component,
{
	fn toggle_speed(&mut self) {
		*self.current_speed = match *self.current_speed {
			SpeedIndex::Default => SpeedIndex::Toggled,
			SpeedIndex::Toggled => SpeedIndex::Default,
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::{
			config::{Config, SpeedIndex},
			movement_path::MovementPath,
		},
		system_param::movement_param::MovementParamMut,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::{
		accessors::get::GetContextMut,
		handles_movement::ConfiguredMovement as MovementMarker,
	};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Motion;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn toggle_speed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(Config::default()).id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let mut ctx =
					MovementParamMut::get_context_mut(&mut p, MovementMarker { entity }).unwrap();
				ctx.toggle_speed();
			})?;

		assert_eq!(
			Some(&SpeedIndex::Toggled),
			app.world().entity(entity).get::<SpeedIndex>(),
		);
		Ok(())
	}

	#[test]
	fn toggle_speed_back() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Config::default(),
				MovementPath::target(Vec3::new(1., 2., 3.)),
			))
			.id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let mut ctx =
					MovementParamMut::get_context_mut(&mut p, MovementMarker { entity }).unwrap();
				ctx.toggle_speed();
				ctx.toggle_speed();
			})?;

		assert_eq!(
			Some(&SpeedIndex::Default),
			app.world().entity(entity).get::<SpeedIndex>(),
		);
		Ok(())
	}
}
