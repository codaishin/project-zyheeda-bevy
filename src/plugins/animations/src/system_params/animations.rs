pub(crate) mod active_animations;
mod get_forward_pitch;
mod get_move_direction;
mod register_animations;

use crate::components::{
	animation_dispatch::AnimationDispatch,
	animation_lookup::AnimationLookup,
	current_forward_pitch::CurrentForwardPitch,
	current_movement_direction::CurrentMovementDirection,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, GetMut},
		handles_animations::{AnimationClips, Animations, WithoutAnimations},
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct AnimationsParamMut<'w, 's, TAnimationGraph = AnimationGraph>
where
	TAnimationGraph: Asset,
{
	commands: ZyheedaCommands<'w, 's>,
	lookups: Query<'w, 's, (), With<AnimationLookup<AnimationClips<AnimationNodeIndex>>>>,
	animators: Query<
		'w,
		's,
		(
			&'static mut AnimationDispatch,
			&'static mut CurrentMovementDirection,
			&'static mut CurrentForwardPitch,
		),
	>,
	graphs: ResMut<'w, Assets<TAnimationGraph>>,
}

impl<TAnimationGraph> GetContextMut<WithoutAnimations>
	for AnimationsParamMut<'static, 'static, TAnimationGraph>
where
	TAnimationGraph: Asset,
{
	type TContext<'ctx> = AnimationsRegisterContextMut<'ctx, TAnimationGraph>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut AnimationsParamMut<TAnimationGraph>,
		WithoutAnimations { entity }: WithoutAnimations,
	) -> Option<Self::TContext<'ctx>> {
		if param.lookups.contains(entity) {
			return None;
		}

		let entity = param.commands.get_mut(&entity)?;
		let graphs = &mut param.graphs;

		Some(AnimationsRegisterContextMut { entity, graphs })
	}
}

impl<TAnimationGraph> GetContextMut<Animations>
	for AnimationsParamMut<'static, 'static, TAnimationGraph>
where
	TAnimationGraph: Asset,
{
	type TContext<'ctx> = AnimationsContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut AnimationsParamMut<TAnimationGraph>,
		Animations { entity }: Animations,
	) -> Option<Self::TContext<'ctx>> {
		if !param.lookups.contains(entity) {
			return None;
		}

		let (dispatch, movement_direction, forward_pitch) = param.animators.get_mut(entity).ok()?;

		Some(AnimationsContextMut {
			dispatch,
			movement_direction,
			forward_pitch,
		})
	}
}

pub struct AnimationsRegisterContextMut<'a, TAnimationGraph = AnimationGraph>
where
	TAnimationGraph: Asset,
{
	entity: ZyheedaEntityCommands<'a>,
	graphs: &'a mut Assets<TAnimationGraph>,
}

pub struct AnimationsContextMut<'a> {
	dispatch: Mut<'a, AnimationDispatch>,
	movement_direction: Mut<'a, CurrentMovementDirection>,
	forward_pitch: Mut<'a, CurrentForwardPitch>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::animation_lookup::AnimationLookup;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	#[derive(TypePath, Asset)]
	struct _Graph;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<Assets<_Graph>>();

		app
	}

	mod without_animations {
		use super::*;

		#[test]
		fn get_context_if_lookup_missing() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn_empty().id();

			let ctx =
				app.world_mut()
					.run_system_once(move |mut p: AnimationsParamMut<_Graph>| {
						AnimationsParamMut::get_context_mut(&mut p, WithoutAnimations { entity })
							.is_some()
					})?;

			assert!(ctx);
			Ok(())
		}

		#[test]
		fn no_context_if_lookup_present() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(AnimationLookup::<AnimationClips<AnimationNodeIndex>>::default())
				.id();

			let ctx =
				app.world_mut()
					.run_system_once(move |mut p: AnimationsParamMut<_Graph>| {
						AnimationsParamMut::get_context_mut(&mut p, WithoutAnimations { entity })
							.is_some()
					})?;

			assert!(!ctx);
			Ok(())
		}
	}

	mod animations {
		use super::*;

		#[test]
		fn no_context_if_lookup_missing() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn(AnimationDispatch::default()).id();

			let ctx =
				app.world_mut()
					.run_system_once(move |mut p: AnimationsParamMut<_Graph>| {
						AnimationsParamMut::get_context_mut(&mut p, Animations { entity }).is_some()
					})?;

			assert!(!ctx);
			Ok(())
		}

		#[test]
		fn get_context_if_lookup_present() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					AnimationLookup::<AnimationClips<AnimationNodeIndex>>::default(),
					AnimationDispatch::default(),
				))
				.id();

			let ctx =
				app.world_mut()
					.run_system_once(move |mut p: AnimationsParamMut<_Graph>| {
						AnimationsParamMut::get_context_mut(&mut p, Animations { entity }).is_some()
					})?;

			assert!(ctx);
			Ok(())
		}
	}
}
