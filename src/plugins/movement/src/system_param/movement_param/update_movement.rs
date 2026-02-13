use crate::system_param::movement_param::MovementContextMut;
use common::{tools::UnitsPerSecond, traits::handles_movement::UpdateMovement};

impl<TMotion> UpdateMovement for MovementContextMut<'_, TMotion> {
	fn update(&mut self, speed: UnitsPerSecond) {
		let Some(movement_definition) = self.movement_definition.as_deref_mut() else {
			return;
		};

		movement_definition.speed = speed;
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::movement_definition::MovementDefinition,
		system_param::movement_param::MovementParamMut,
	};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::{
		tools::Units,
		traits::{accessors::get::GetContextMut, handles_movement::Movement},
	};
	use testing::SingleThreadedApp;

	struct _Motion;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_movement_definition() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(MovementDefinition {
				radius: Units::from(42.),
				speed: UnitsPerSecond::from(11.),
			})
			.id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let mut ctx =
					MovementParamMut::get_context_mut(&mut p, Movement { entity }).unwrap();
				ctx.update(UnitsPerSecond::from(110.));
			})?;

		assert_eq!(
			Some(&MovementDefinition {
				radius: Units::from(42.),
				speed: UnitsPerSecond::from(110.),
			}),
			app.world().entity(entity).get::<MovementDefinition>(),
		);
		Ok(())
	}
}
