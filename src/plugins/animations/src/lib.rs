pub mod components;
pub mod systems;
pub mod traits;

mod resource;

use crate::systems::{
	discover_animation_mask_bones::DiscoverMaskChains,
	init_animation_clips::InitAnimationClips,
	init_animation_graph::InitAnimationGraph,
	mask_animation_nodes::MaskAnimationNodes,
	remove_unused_animation_targets::RemoveUnusedAnimationTargets,
};
use bevy::prelude::*;
use common::{
	labels::Labels,
	systems::{
		insert_required::{InsertOn, InsertRequired},
		log::log,
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
use resource::AnimationData;
use systems::play_animation_clip::PlayAnimationClip;

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
			Startup,
			(
				TAgent::init_animation_clips::<AnimationGraph, AssetServer>,
				TAgent::mask_animation_nodes.pipe(log),
			)
				.chain(),
		)
		.add_systems(
			Labels::PREFAB_INSTANTIATION.label(),
			(
				InsertOn::<TAgent>::required::<AnimationDispatch>(dispatch),
				TAgent::init_animation_graph_and_transitions::<AnimationDispatch>,
				TAgent::set_animation_mask_bones,
				TAgent::remove_unused_animation_targets,
			)
				.chain(),
		)
		.add_systems(
			Update,
			AnimationDispatch::play_animation_clip_via::<&mut AnimationPlayer, TAgent>,
		);
	}

	fn register_movement_direction<TMovementDirection>(_: &mut App)
	where
		TMovementDirection: Component + GetMovementDirection,
	{
	}
}

impl HasAnimationsDispatch for AnimationsPlugin {
	type TAnimationDispatch = AnimationDispatch;
}

impl Plugin for AnimationsPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			PostUpdate,
			(
				AnimationDispatch::track_in_self_and_children::<AnimationPlayer>().system(),
				AnimationDispatch::track_in_self_and_children::<AnimationTransitions>().system(),
			),
		);
	}
}
