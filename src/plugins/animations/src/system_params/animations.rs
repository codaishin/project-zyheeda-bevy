pub(crate) mod active_animations;
mod get_forward_pitch;
mod get_move_direction;
mod register_animations;

use crate::components::{
	animation_dispatch::AnimationDispatch,
	current_forward_pitch::CurrentForwardPitch,
	current_movement_direction::CurrentMovementDirection,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, GetMut},
		handles_animations::{Animations, WithoutAnimations},
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct AnimationsParamMut<'w, 's, TAnimationGraph = AnimationGraph>
where
	TAnimationGraph: Asset,
{
	commands: ZyheedaCommands<'w, 's>,
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
	for AnimationsParamMut<'_, '_, TAnimationGraph>
where
	TAnimationGraph: Asset,
{
	type TContext<'ctx> = AnimationsRegisterContextMut<'ctx, TAnimationGraph>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut AnimationsParamMut<TAnimationGraph>,
		WithoutAnimations { entity }: WithoutAnimations,
	) -> Option<Self::TContext<'ctx>> {
		if param.animators.contains(entity) {
			return None;
		}

		let entity = param.commands.get_mut(&entity)?;
		let graphs = &mut param.graphs;

		Some(AnimationsRegisterContextMut { entity, graphs })
	}
}

impl<TAnimationGraph> GetContextMut<Animations> for AnimationsParamMut<'_, '_, TAnimationGraph>
where
	TAnimationGraph: Asset,
{
	type TContext<'ctx> = AnimationsContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut AnimationsParamMut<TAnimationGraph>,
		animations: Animations,
	) -> Option<Self::TContext<'ctx>> {
		let (dispatch, movement_direction, forward_pitch) =
			param.animators.get_mut(animations.entity).ok()?;

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
