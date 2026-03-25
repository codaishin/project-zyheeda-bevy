pub(crate) mod context_changed;
mod current_movement;
mod start_movement;
mod stop_movement;

use crate::system_param::movement_param::context_changed::JustRemovedMovements;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContext, GetContextMut, GetMut},
		handles_movement::Movement,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct MovementParam<'w, 's, TMotion>
where
	TMotion: Component,
{
	movements: Query<'w, 's, Ref<'static, TMotion>>,
	just_removed_movements: Res<'w, JustRemovedMovements>,
}

impl<TMotion> GetContext<Movement> for MovementParam<'_, '_, TMotion>
where
	TMotion: Component,
{
	type TContext<'ctx> = MovementContext<'ctx, TMotion>;

	fn get_context<'ctx>(
		param: &'ctx MovementParam<TMotion>,
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
pub struct MovementParamMut<'w, 's, TMotion>
where
	TMotion: Component,
{
	commands: ZyheedaCommands<'w, 's>,
	motions: Query<'w, 's, &'static TMotion>,
}

impl<TMotion> GetContextMut<Movement> for MovementParamMut<'_, '_, TMotion>
where
	TMotion: Component,
{
	type TContext<'ctx> = MovementContextMut<'ctx, TMotion>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut MovementParamMut<TMotion>,
		Movement { entity }: Movement,
	) -> Option<Self::TContext<'ctx>> {
		let motion = param.motions.get(entity).ok();
		let entity = param.commands.get_mut(&entity)?;

		Some(MovementContextMut { entity, motion })
	}
}

pub enum MovementContext<'ctx, TMotion>
where
	TMotion: Component,
{
	Movement(Ref<'ctx, TMotion>),
	JustRemoved,
	Empty,
}

pub struct MovementContextMut<'ctx, TMotion>
where
	TMotion: Component,
{
	entity: ZyheedaEntityCommands<'ctx>,
	motion: Option<&'ctx TMotion>,
}
