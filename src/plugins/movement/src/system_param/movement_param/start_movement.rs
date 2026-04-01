use crate::{components::movement::Movement, system_param::movement_param::MovementContextMut};
use bevy::ecs::component::Component;
use common::traits::handles_movement::{MovementTarget, StartMovement};

impl<TMotion> StartMovement for MovementContextMut<'_, TMotion>
where
	TMotion: Component,
{
	fn start<T>(&mut self, target: T)
	where
		T: Into<MovementTarget>,
	{
		self.entity.try_insert(Movement::from(target));
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{components::config::Config, system_param::movement_param::MovementParamMut};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{
		accessors::get::GetContextMut,
		handles_movement::ConfiguredMovement,
		thread_safe::ThreadSafe,
	};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Motion;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test_case(Vec3::new(1.,2.,3.); "to point")]
	#[test_case(Dir3::NEG_X; "towards direction")]
	fn insert_path(
		target: impl Into<MovementTarget> + Copy + ThreadSafe,
	) -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(Config::default()).id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let mut ctx =
					MovementParamMut::get_context_mut(&mut p, ConfiguredMovement { entity })
						.unwrap();
				ctx.start(target);
			})?;

		assert_eq!(
			Some(&Movement::from(target)),
			app.world().entity(entity).get::<Movement>(),
		);
		Ok(())
	}
}
