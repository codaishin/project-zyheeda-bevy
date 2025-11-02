mod start_movement;
mod stop_movement;
mod update_movement;

use std::marker::PhantomData;

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
pub struct MovementParamMut<'w, 's, TMotion>
where
	TMotion: 'static,
{
	commands: ZyheedaCommands<'w, 's>,
	movement_definitions: Query<'w, 's, &'static mut MovementDefinition>,
	_p: PhantomData<TMotion>,
}

impl<TMotion> EntityContextMut<Movement> for MovementParamMut<'_, '_, TMotion> {
	type TContext<'ctx> = MovementContextMut<'ctx, TMotion>;

	fn get_entity_context_mut<'ctx>(
		param: &'ctx mut MovementParamMut<TMotion>,
		entity: Entity,
		_: Movement,
	) -> Option<Self::TContext<'ctx>> {
		let movement_definition = param.movement_definitions.get_mut(entity).ok();
		let entity = param.commands.get_mut(&entity)?;

		Some(MovementContextMut {
			entity,
			movement_definition,
			_p: PhantomData,
		})
	}
}

pub struct MovementContextMut<'ctx, TMotion> {
	entity: ZyheedaEntityCommands<'ctx>,
	movement_definition: Option<Mut<'ctx, MovementDefinition>>,
	_p: PhantomData<TMotion>,
}
