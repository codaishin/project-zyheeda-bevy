use crate::{components::movement::Movement, system_param::movement_param::MovementContextMut};
use bevy::ecs::component::Component;
use common::traits::handles_movement::StopMovement;

impl<TMotion> StopMovement for MovementContextMut<'_, TMotion>
where
	TMotion: Component,
{
	fn stop(&mut self) {
		self.entity.try_insert(Movement::None);
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{components::config::Config, system_param::movement_param::MovementParamMut};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{accessors::get::GetContextMut, handles_movement::ConfiguredMovement};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Motion;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_movement_none() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(Config::default()).id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let mut ctx =
					MovementParamMut::get_context_mut(&mut p, ConfiguredMovement { entity })
						.unwrap();
				ctx.stop();
			})?;

		assert_eq!(
			Some(&Movement::None),
			app.world().entity(entity).get::<Movement>(),
		);
		Ok(())
	}
}
