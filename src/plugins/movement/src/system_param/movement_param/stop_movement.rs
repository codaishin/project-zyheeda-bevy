use crate::{
	components::{movement_path::MovementPath, ongoing_movement::OngoingMovement},
	system_param::movement_param::MovementContextMut,
};
use bevy::ecs::component::Component;
use common::traits::handles_movement::StopMovement;

impl<TMotion> StopMovement for MovementContextMut<'_, TMotion>
where
	TMotion: Component,
{
	fn stop(&mut self) {
		self.entity
			.try_insert((MovementPath::stop(), OngoingMovement::Stop));
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::{config::Config, movement_path::MovementPath},
		system_param::movement_param::MovementParamMut,
	};
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
	fn insert_path_stop() -> Result<(), RunSystemError> {
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
					MovementParamMut::get_context_mut(&mut p, ConfiguredMovement { entity })
						.unwrap();
				ctx.stop();
			})?;

		assert_eq!(
			Some(&MovementPath::stop()),
			app.world().entity(entity).get::<MovementPath>(),
		);
		Ok(())
	}

	#[test]
	fn insert_movement_stop() -> Result<(), RunSystemError> {
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
					MovementParamMut::get_context_mut(&mut p, ConfiguredMovement { entity })
						.unwrap();
				ctx.stop();
			})?;

		assert_eq!(
			Some(&OngoingMovement::Stop),
			app.world().entity(entity).get::<OngoingMovement>(),
		);
		Ok(())
	}
}
