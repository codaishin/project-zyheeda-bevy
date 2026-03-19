pub(crate) mod context_changed;
mod current_movement;
mod start_movement;
mod stop_movement;
mod update_movement;

use crate::{
	components::{
		movement::path_or_direction::PathOrDirection,
		movement_definition::MovementDefinition,
	},
	system_param::movement_param::context_changed::JustRemovedMovements,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContext, GetContextMut, GetMut},
		handles_movement::Movement,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct MovementParam<'w, 's> {
	movements: Query<'w, 's, Ref<'static, PathOrDirection>>,
	just_removed_movements: Res<'w, JustRemovedMovements>,
}

impl GetContext<Movement> for MovementParam<'_, '_> {
	type TContext<'ctx> = MovementContext<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx MovementParam,
		Movement { entity }: Movement,
	) -> Option<Self::TContext<'ctx>> {
		let ctx = match param.movements.get(entity) {
			Ok(movement) => MovementContext::Movement(movement),
			_ if param.just_removed_movements.0.contains(&entity) => MovementContext::JustRemoved,
			_ => MovementContext::Empty,
		};

		Some(ctx)
	}
}

#[derive(SystemParam)]
pub struct MovementParamMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
	movement_definitions: Query<'w, 's, &'static mut MovementDefinition>,
	movements: Query<'w, 's, &'static PathOrDirection>,
}

impl GetContextMut<Movement> for MovementParamMut<'_, '_> {
	type TContext<'ctx> = MovementContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut MovementParamMut,
		Movement { entity }: Movement,
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

pub enum MovementContext<'ctx> {
	Movement(Ref<'ctx, PathOrDirection>),
	JustRemoved,
	Empty,
}

pub struct MovementContextMut<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
	movement_definition: Option<Mut<'ctx, MovementDefinition>>,
	movement: Option<&'ctx PathOrDirection>,
}
