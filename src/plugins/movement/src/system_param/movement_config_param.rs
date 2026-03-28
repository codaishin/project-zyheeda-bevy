mod configure_movement;

use crate::components::config::Config;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, GetMut},
		handles_movement::NotConfiguredMovement,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct MovementConfigParamMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
	not_configure: Query<'w, 's, (), Without<Config>>,
}

impl GetContextMut<NotConfiguredMovement> for MovementConfigParamMut<'_, '_> {
	type TContext<'ctx> = MovementConfigContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut bevy::ecs::system::SystemParamItem<Self>,
		NotConfiguredMovement { entity }: NotConfiguredMovement,
	) -> Option<Self::TContext<'ctx>> {
		if !param.not_configure.contains(entity) {
			return None;
		}

		Some(MovementConfigContextMut {
			entity: param.commands.get_mut(&entity)?,
		})
	}
}

pub struct MovementConfigContextMut<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::config::Config;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn get_context() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		let ctx = app
			.world_mut()
			.run_system_once(move |mut p: MovementConfigParamMut| {
				let ctx = MovementConfigParamMut::get_context_mut(
					&mut p,
					NotConfiguredMovement { entity },
				);
				ctx.is_some()
			})?;

		assert!(ctx);
		Ok(())
	}

	#[test]
	fn get_no_context_when_configured() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(Config::default()).id();

		let ctx = app
			.world_mut()
			.run_system_once(move |mut p: MovementConfigParamMut| {
				let ctx = MovementConfigParamMut::get_context_mut(
					&mut p,
					NotConfiguredMovement { entity },
				);
				ctx.is_some()
			})?;

		assert!(!ctx);
		Ok(())
	}
}
