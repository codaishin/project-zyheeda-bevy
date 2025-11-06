mod current_movement;
mod start_movement;
mod stop_movement;
mod update_movement;

use crate::components::{
	movement::{Movement, path_or_direction::PathOrDirection},
	movement_definition::MovementDefinition,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, GetMut},
		handles_movement::Movement as MovementMarker,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct MovementParamMut<'w, 's, TMotion>
where
	TMotion: ThreadSafe,
{
	commands: ZyheedaCommands<'w, 's>,
	movement_definitions: Query<'w, 's, &'static mut MovementDefinition>,
	movements: Query<'w, 's, &'static Movement<PathOrDirection<TMotion>>>,
}

impl<TMotion> GetContextMut<MovementMarker> for MovementParamMut<'_, '_, TMotion>
where
	TMotion: ThreadSafe,
{
	type TContext<'ctx> = MovementContextMut<'ctx, TMotion>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut MovementParamMut<TMotion>,
		MovementMarker { entity }: MovementMarker,
	) -> Option<Self::TContext<'ctx>> {
		let movement_definition = param.movement_definitions.get_mut(entity).ok();
		let movement = param.movements.get(entity).ok();
		let entity = param.commands.get_mut(&entity)?;

		Some(MovementContextMut {
			entity,
			movement_definition,
			movement,
		})
	}
}

pub struct MovementContextMut<'ctx, TMotion>
where
	TMotion: ThreadSafe,
{
	entity: ZyheedaEntityCommands<'ctx>,
	movement_definition: Option<Mut<'ctx, MovementDefinition>>,
	movement: Option<&'ctx Movement<PathOrDirection<TMotion>>>,
}
