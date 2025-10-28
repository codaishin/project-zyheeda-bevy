use std::marker::PhantomData;

use crate::components::movement::{Movement, path_or_wasd::PathOrWasd};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{EntityContext, EntityContextMut, GetMut},
		handles_movement_behavior::{Movement as MovementMarker, PathMotionSpec, SetMovement},
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct ReadMovement<'w, 's, TMotion>
where
	TMotion: ThreadSafe,
{
	movements: Query<'w, 's, Ref<'static, Movement<PathOrWasd<TMotion>>>>,
}

impl<'w, 's, TMotion> EntityContext<MovementMarker> for ReadMovement<'w, 's, TMotion>
where
	TMotion: ThreadSafe,
{
	type TContext<'ctx> = Ref<'ctx, Movement<PathOrWasd<TMotion>>>;

	fn get_entity_context<'ctx>(
		param: &'ctx ReadMovement<TMotion>,
		entity: Entity,
		_: MovementMarker,
	) -> Option<Self::TContext<'ctx>> {
		param.movements.get(entity).ok()
	}
}

#[derive(SystemParam)]
pub struct WriteMovement<'w, 's, TMotion>
where
	TMotion: ThreadSafe,
{
	commands: ZyheedaCommands<'w, 's>,
	_p: PhantomData<TMotion>,
}

impl<'w, 's, TMotion> EntityContextMut<MovementMarker> for WriteMovement<'w, 's, TMotion>
where
	TMotion: ThreadSafe,
{
	type TContext<'ctx> = InsertMovement<'ctx, TMotion>;

	fn get_entity_context_mut<'ctx>(
		param: &'ctx mut WriteMovement<TMotion>,
		entity: Entity,
		_: MovementMarker,
	) -> Option<Self::TContext<'ctx>> {
		let entity = param.commands.get_mut(&entity)?;

		Some(InsertMovement {
			entity,
			_p: PhantomData,
		})
	}
}

pub struct InsertMovement<'a, TMotion> {
	entity: ZyheedaEntityCommands<'a>,
	_p: PhantomData<TMotion>,
}

impl<'a, TMotion> SetMovement for InsertMovement<'a, TMotion>
where
	TMotion: ThreadSafe,
{
	fn set_movement(&mut self, spec: PathMotionSpec) {
		self.entity
			.try_insert(Movement::<PathOrWasd<TMotion>>::from(spec));
	}
}
