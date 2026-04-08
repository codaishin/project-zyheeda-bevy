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
pub struct AnimationsParamMut<
	'w,
	's,
	TAnimationServer = AssetServer,
	TAnimationGraph = AnimationGraph,
> where
	TAnimationServer: Resource,
	TAnimationGraph: Asset,
{
	commands: ZyheedaCommands<'w, 's>,
	asset_server: ResMut<'w, TAnimationServer>,
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

impl<TAnimationServer, TAnimationGraph> GetContextMut<WithoutAnimations>
	for AnimationsParamMut<'_, '_, TAnimationServer, TAnimationGraph>
where
	TAnimationServer: Resource,
	TAnimationGraph: Asset,
{
	type TContext<'ctx> = AnimationsRegisterContextMut<'ctx, TAnimationServer, TAnimationGraph>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut AnimationsParamMut<TAnimationServer, TAnimationGraph>,
		WithoutAnimations { entity }: WithoutAnimations,
	) -> Option<Self::TContext<'ctx>> {
		if param.animators.contains(entity) {
			return None;
		}

		let entity = param.commands.get_mut(&entity)?;
		let asset_server = &mut param.asset_server;
		let graphs = &mut param.graphs;

		Some(AnimationsRegisterContextMut {
			entity,
			asset_server,
			graphs,
		})
	}
}

impl<TAnimationServer, TAnimationGraph> GetContextMut<Animations>
	for AnimationsParamMut<'_, '_, TAnimationServer, TAnimationGraph>
where
	TAnimationServer: Resource,
	TAnimationGraph: Asset,
{
	type TContext<'ctx> = AnimationsContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut AnimationsParamMut<TAnimationServer, TAnimationGraph>,
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

pub struct AnimationsRegisterContextMut<
	'a,
	TLoadAnimations = AssetServer,
	TAnimationGraph = AnimationGraph,
> where
	TAnimationGraph: Asset,
{
	entity: ZyheedaEntityCommands<'a>,
	asset_server: &'a mut TLoadAnimations,
	graphs: &'a mut Assets<TAnimationGraph>,
}

pub struct AnimationsContextMut<'a> {
	dispatch: Mut<'a, AnimationDispatch>,
	movement_direction: Mut<'a, CurrentMovementDirection>,
	forward_pitch: Mut<'a, CurrentForwardPitch>,
}
