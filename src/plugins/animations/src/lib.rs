pub mod components;
pub mod systems;
pub mod traits;

#[cfg(test)]
pub(crate) mod test_tools;

use crate::systems::{
	discover_animation_mask_bones::DiscoverMaskChains,
	init_animation_components::InitAnimationComponents,
	mask_animation_nodes::MaskAnimationNodes,
	remove_unused_animation_targets::RemoveUnusedAnimationTargets,
	set_directional_animation_weights::SetDirectionalAnimationWeights,
};
use bevy::prelude::*;
use common::{
	labels::Labels,
	systems::{
		insert_required::{InsertOn, InsertRequired},
		track_components::TrackComponentInSelfAndChildren,
	},
	traits::animation::{
		AnimationMaskDefinition,
		ConfigureNewAnimationDispatch,
		GetAnimationDefinitions,
		GetMovementDirection,
		HasAnimationsDispatch,
		RegisterAnimations,
	},
};
use components::animation_dispatch::AnimationDispatch;
use systems::{
	init_player_components::InitPlayerComponents,
	play_animation_clip::PlayAnimationClip,
};

pub struct AnimationsPlugin;

impl RegisterAnimations for AnimationsPlugin {
	fn register_animations<TAgent>(app: &mut App)
	where
		TAgent: Component + GetAnimationDefinitions + ConfigureNewAnimationDispatch,
		for<'a> AnimationMask: From<&'a TAgent::TAnimationMask>,
		for<'a> AnimationMaskDefinition: From<&'a TAgent::TAnimationMask>,
	{
		let dispatch = |agent: &TAgent| {
			let mut dispatch = AnimationDispatch::default();
			TAgent::configure_animation_dispatch(agent, &mut dispatch);
			dispatch
		};

		app.add_systems(
			Labels::PREFAB_INSTANTIATION.label(),
			(
				InsertOn::<TAgent>::required::<AnimationDispatch>(dispatch),
				TAgent::init_animation_components::<AnimationGraph, AssetServer>,
				TAgent::mask_animation_nodes,
				TAgent::set_animation_mask_bones,
				TAgent::remove_unused_animation_targets,
			)
				.chain(),
		);
	}

	fn register_movement_direction<TMovementDirection>(app: &mut App)
	where
		TMovementDirection: Component + GetMovementDirection,
	{
		app.add_systems(
			Update,
			AnimationDispatch::set_directional_animation_weights::<TMovementDirection>,
		);
	}
}

impl HasAnimationsDispatch for AnimationsPlugin {
	type TAnimationDispatch = AnimationDispatch;
}

impl Plugin for AnimationsPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			AnimationDispatch::play_animation_clip_via::<&mut AnimationPlayer>,
		)
		.add_systems(
			PostUpdate,
			(
				AnimationDispatch::track_in_self_and_children::<AnimationPlayer>().system(),
				AnimationDispatch::track_in_self_and_children::<AnimationGraphHandle>().system(),
				AnimationDispatch::init_player_components::<AnimationGraphHandle>,
			),
		);
	}
}
