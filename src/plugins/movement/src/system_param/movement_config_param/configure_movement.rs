use crate::{
	components::config::Config,
	system_param::movement_config_param::MovementConfigContextMut,
};
use common::traits::handles_movement::{ConfigureMovement, MovementSpeed, RequiredClearance};

impl ConfigureMovement for MovementConfigContextMut<'_> {
	fn configure(&mut self, speed: MovementSpeed, required_clearance: RequiredClearance) {
		self.entity.try_insert(Config {
			speed,
			required_clearance,
		});
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::system_param::movement_config_param::MovementConfigParamMut;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{
		tools::{Units, UnitsPerSecond},
		traits::{accessors::get::GetContextMut, handles_movement::NotConfiguredMovement},
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn toggle_speed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: MovementConfigParamMut| {
				let mut ctx = MovementConfigParamMut::get_context_mut(
					&mut p,
					NotConfiguredMovement { entity },
				)
				.unwrap();
				ctx.configure(
					MovementSpeed::Fixed(UnitsPerSecond::from_u8(11)),
					RequiredClearance {
						vertical: Units::from_u8(5),
						horizontal: Units::from_u8(2),
					},
				);
			})?;

		assert_eq!(
			Some(&Config {
				speed: MovementSpeed::Fixed(UnitsPerSecond::from_u8(11)),
				required_clearance: RequiredClearance {
					vertical: Units::from_u8(5),
					horizontal: Units::from_u8(2),
				},
			}),
			app.world().entity(entity).get::<Config>(),
		);
		Ok(())
	}
}
