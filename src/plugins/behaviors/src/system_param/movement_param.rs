mod start_movement;
mod stop_movement;
mod update_movement;

use crate::components::movement_definition::MovementDefinition;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{EntityContextMut, GetMut},
		handles_movement::Movement,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct MovementParamMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
	movement_definitions: Query<'w, 's, &'static mut MovementDefinition>,
}

impl EntityContextMut<Movement> for MovementParamMut<'_, '_> {
	type TContext<'ctx> = MovementContextMut<'ctx>;

	fn get_entity_context_mut<'ctx>(
		param: &'ctx mut MovementParamMut,
		entity: Entity,
		_: Movement,
	) -> Option<Self::TContext<'ctx>> {
		let movement_definition = param.movement_definitions.get_mut(entity).ok();
		let entity = param.commands.get_mut(&entity)?;

		Some(MovementContextMut {
			entity,
			movement_definition,
		})
	}
}

pub struct MovementContextMut<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
	movement_definition: Option<Mut<'ctx, MovementDefinition>>,
}
