pub(crate) mod context_changed;
mod current_movement;
mod start_movement;
mod stop_movement;
mod update_movement;

use crate::{
	components::{
		movement::{Movement, path_or_direction::PathOrDirection},
		movement_definition::MovementDefinition,
	},
	system_param::movement_param::context_changed::JustRemovedMovements,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContext, GetContextMut, GetMut},
		handles_movement::Movement as MovementMarker,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct MovementParam<'w, 's, TMotion>
where
	TMotion: ThreadSafe,
{
	movements: Query<'w, 's, Ref<'static, Movement<PathOrDirection<TMotion>>>>,
	just_removed_movements: Res<'w, JustRemovedMovements>,
}

impl<TMotion> GetContext<MovementMarker> for MovementParam<'_, '_, TMotion>
where
	TMotion: ThreadSafe,
{
	type TContext<'ctx> = MovementContext<'ctx, TMotion>;

	fn get_context<'ctx>(
		param: &'ctx MovementParam<TMotion>,
		MovementMarker { entity }: MovementMarker,
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

pub enum MovementContext<'ctx, TMotion>
where
	TMotion: ThreadSafe,
{
	Movement(Ref<'ctx, Movement<PathOrDirection<TMotion>>>),
	JustRemoved,
	Empty,
}

pub struct MovementContextMut<'ctx, TMotion>
where
	TMotion: ThreadSafe,
{
	entity: ZyheedaEntityCommands<'ctx>,
	movement_definition: Option<Mut<'ctx, MovementDefinition>>,
	movement: Option<&'ctx Movement<PathOrDirection<TMotion>>>,
}
