pub(crate) mod override_animations;
mod register_animations;
mod set_movement_direction;

use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, GetMut},
		animation::Animations,
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
	graphs: ResMut<'w, Assets<TAnimationGraph>>,
}

impl<TAnimationServer, TAnimationGraph> GetContextMut<Animations>
	for AnimationsParamMut<'_, '_, TAnimationServer, TAnimationGraph>
where
	TAnimationServer: Resource,
	TAnimationGraph: Asset,
{
	type TContext<'ctx> = AnimationsContextMut<'ctx, TAnimationServer, TAnimationGraph>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut AnimationsParamMut<TAnimationServer, TAnimationGraph>,
		animations: Animations,
	) -> Option<Self::TContext<'ctx>> {
		let entity = param.commands.get_mut(&animations.entity)?;
		let asset_server = &mut param.asset_server;
		let graphs = &mut param.graphs;

		Some(AnimationsContextMut {
			entity,
			asset_server,
			graphs,
		})
	}
}

pub struct AnimationsContextMut<'a, TLoadAnimations = AssetServer, TAnimationGraph = AnimationGraph>
where
	TAnimationGraph: Asset,
{
	entity: ZyheedaEntityCommands<'a>,
	asset_server: &'a mut TLoadAnimations,
	graphs: &'a mut Assets<TAnimationGraph>,
}
