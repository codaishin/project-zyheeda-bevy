mod body;
mod default_attributes;

use crate::components::{body::Body, default_attributes::DefaultAttributes};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetMut, TryGetContextMut},
		handles_physics::{NoBodyConfigured, NoDefaultAttributes},
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct ConfigParamMut<'w, 's> {
	default_attributes: Query<'w, 's, (), With<DefaultAttributes>>,
	bodies: Query<'w, 's, (), With<Body>>,
	commands: ZyheedaCommands<'w, 's>,
}

impl TryGetContextMut<NoDefaultAttributes> for ConfigParamMut<'static, 'static> {
	type TContext<'ctx> = ConfigContextMut<'ctx>;

	fn try_get_context_mut<'ctx>(
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

impl TryGetContextMut<NoBodyConfigured> for ConfigParamMut<'static, 'static> {
	type TContext<'ctx> = ConfigContextMut<'ctx>;

	fn try_get_context_mut<'ctx>(
		param: &'ctx mut bevy::ecs::system::SystemParamItem<Self>,
		NoBodyConfigured { entity }: NoBodyConfigured,
	) -> Option<Self::TContext<'ctx>> {
		if param.bodies.contains(entity) {
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
	use common::traits::handles_physics::{PhysicalDefaultAttributes, physical_bodies::BodyConfig};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	mod default_attributes {
		use super::*;

		#[test]
		fn get_config() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn_empty().id();

			let ctx_entity = app
				.world_mut()
				.run_system_once(move |mut p: ConfigParamMut| {
					let key = NoDefaultAttributes { entity };
					let ctx = ConfigParamMut::try_get_context_mut(&mut p, key);
					ctx.map(|c| c.entity.id())
				})?;

			assert_eq!(Some(entity), ctx_entity);
			Ok(())
		}

		#[test]
		fn get_no_config_when_component_present() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(DefaultAttributes(PhysicalDefaultAttributes::default()))
				.id();

			let ctx_entity = app
				.world_mut()
				.run_system_once(move |mut p: ConfigParamMut| {
					let key = NoDefaultAttributes { entity };
					let ctx = ConfigParamMut::try_get_context_mut(&mut p, key);
					ctx.map(|c| c.entity.id())
				})?;

			assert_eq!(None, ctx_entity);
			Ok(())
		}
	}

	mod body {
		use super::*;

		#[test]
		fn get_config() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn_empty().id();

			let ctx_entity = app
				.world_mut()
				.run_system_once(move |mut p: ConfigParamMut| {
					let key = NoBodyConfigured { entity };
					let ctx = ConfigParamMut::try_get_context_mut(&mut p, key);
					ctx.map(|c| c.entity.id())
				})?;

			assert_eq!(Some(entity), ctx_entity);
			Ok(())
		}

		#[test]
		fn get_no_config_when_component_present() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn(Body(BodyConfig::default())).id();

			let ctx_entity = app
				.world_mut()
				.run_system_once(move |mut p: ConfigParamMut| {
					let key = NoBodyConfigured { entity };
					let ctx = ConfigParamMut::try_get_context_mut(&mut p, key);
					ctx.map(|c| c.entity.id())
				})?;

			assert_eq!(None, ctx_entity);
			Ok(())
		}
	}
}
