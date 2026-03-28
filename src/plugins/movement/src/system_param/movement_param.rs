pub(crate) mod context_changed;
mod current_movement;
mod start_movement;
mod stop_movement;
mod toggle_speed;

use crate::{
	components::config::{Config, SpeedIndex},
	system_param::movement_param::context_changed::JustRemovedMovements,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContext, GetContextMut, GetMut},
		handles_movement::{ConfiguredMovement, Movement},
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct MovementParam<'w, 's, TMotion>
where
	TMotion: Component,
{
	movements: Query<'w, 's, Ref<'static, TMotion>>,
	speed_index: Query<'w, 's, Ref<'static, SpeedIndex>>,
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
		let motion = match param.movements.get(entity) {
			Ok(movement) => MotionState::Movement(movement),
			_ if param.just_removed_movements.0.contains(&entity) => MotionState::JustRemoved,
			_ => MotionState::Empty,
		};
		let speed_index = param.speed_index.get(entity).ok();

		Some(MovementContext {
			motion,
			current_speed: speed_index,
		})
	}
}

#[derive(SystemParam)]
pub struct MovementParamMut<'w, 's, TMotion>
where
	TMotion: Component,
{
	commands: ZyheedaCommands<'w, 's>,
	motions: Query<
		'w,
		's,
		(
			Option<&'static TMotion>,
			&'static Config,
			&'static mut SpeedIndex,
		),
	>,
}

impl<TMotion> GetContextMut<ConfiguredMovement> for MovementParamMut<'_, '_, TMotion>
where
	TMotion: Component,
{
	type TContext<'ctx> = MovementContextMut<'ctx, TMotion>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut MovementParamMut<TMotion>,
		ConfiguredMovement { entity }: ConfiguredMovement,
	) -> Option<Self::TContext<'ctx>> {
		let (motion, config, current_speed) = param.motions.get_mut(entity).ok()?;
		let entity = param.commands.get_mut(&entity)?;

		Some(MovementContextMut {
			entity,
			motion,
			config,
			current_speed,
		})
	}
}

pub struct MovementContext<'ctx, TMotion>
where
	TMotion: Component,
{
	motion: MotionState<'ctx, TMotion>,
	current_speed: Option<Ref<'ctx, SpeedIndex>>,
}

pub(crate) enum MotionState<'ctx, TMotion>
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
	config: &'ctx Config,
	current_speed: Mut<'ctx, SpeedIndex>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::config::Config;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Motion;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn get_context() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(Config::default()).id();

		let ctx = app
			.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let ctx = MovementParamMut::<_Motion>::get_context_mut(
					&mut p,
					ConfiguredMovement { entity },
				);
				ctx.is_some()
			})?;

		assert!(ctx);
		Ok(())
	}

	#[test]
	fn get_no_context_when_not_configured() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		let ctx = app
			.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let ctx = MovementParamMut::<_Motion>::get_context_mut(
					&mut p,
					ConfiguredMovement { entity },
				);
				ctx.is_some()
			})?;

		assert!(!ctx);
		Ok(())
	}
}
