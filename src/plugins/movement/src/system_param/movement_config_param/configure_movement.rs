use crate::{
	components::config::Config,
	system_param::movement_config_param::MovementConfigContextMut,
};
use bevy::prelude::*;
use common::{
	tools::Units,
	traits::handles_movement::{ConfigureMovement, MovementSpeed},
};

impl ConfigureMovement for MovementConfigContextMut<'_> {
	fn configure(&mut self, speed: MovementSpeed, required_clearance: Units, ground_offset: Vec3) {
		self.entity.try_insert(Config {
			speed,
			required_clearance,
			ground_offset,
		});
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::system_param::movement_config_param::MovementConfigParamMut;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		tools::UnitsPerSecond,
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
					Units::from_u8(2),
					Vec3::new(1., 2., 3.),
				);
			})?;

		assert_eq!(
			Some(&Config {
				speed: MovementSpeed::Fixed(UnitsPerSecond::from_u8(11)),
				required_clearance: Units::from_u8(2),
				ground_offset: Vec3::new(1., 2., 3.),
			}),
			app.world().entity(entity).get::<Config>(),
		);
		Ok(())
	}
}
