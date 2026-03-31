mod default_attributes;

use crate::components::default_attributes::DefaultAttributes;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, GetMut},
		handles_physics::NoDefaultAttributes,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct ConfigParamMut<'w, 's> {
	default_attributes: Query<'w, 's, (), With<DefaultAttributes>>,
	commands: ZyheedaCommands<'w, 's>,
}

impl GetContextMut<NoDefaultAttributes> for ConfigParamMut<'_, '_> {
	type TContext<'ctx> = ConfigContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut ConfigParamMut,
		NoDefaultAttributes { entity }: NoDefaultAttributes,
	) -> Option<Self::TContext<'ctx>> {
		if param.default_attributes.contains(entity) {
			return None;
		}

		Some(ConfigContextMut {
			entity: param.commands.get_mut(&entity)?,
		})
	}
}

pub struct ConfigContextMut<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::handles_physics::PhysicalDefaultAttributes;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn get_attribute_config() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		let ctx_entity = app
			.world_mut()
			.run_system_once(move |mut p: ConfigParamMut| {
				let key = NoDefaultAttributes { entity };
				let ctx = ConfigParamMut::get_context_mut(&mut p, key);
				ctx.map(|c| c.entity.id())
			})?;

		assert_eq!(Some(entity), ctx_entity);
		Ok(())
	}

	#[test]
	fn get_no_attribute_config_when_default_present() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(DefaultAttributes(PhysicalDefaultAttributes::default()))
			.id();

		let ctx_entity = app
			.world_mut()
			.run_system_once(move |mut p: ConfigParamMut| {
				let key = NoDefaultAttributes { entity };
				let ctx = ConfigParamMut::get_context_mut(&mut p, key);
				ctx.map(|c| c.entity.id())
			})?;

		assert_eq!(None, ctx_entity);
		Ok(())
	}
}
