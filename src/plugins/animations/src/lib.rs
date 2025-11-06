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
	systems::track_components::TrackComponentInSelfAndChildren,
	traits::{
		animation::{
			AffectedAnimationBones,
			ConfigureNewAnimationDispatch,
			GetAnimationDefinitions,
			GetMovementDirection,
			HasAnimationsDispatch,
			RegisterAnimations,
		},
		handles_saving::HandlesSaving,
		register_derived_component::RegisterDerivedComponent,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::animation_dispatch::AnimationDispatch;
use std::marker::PhantomData;
use systems::{
	init_player_components::InitPlayerComponents,
	play_animation_clip::PlayAnimationClip,
};

pub struct AnimationsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSavegame> AnimationsPlugin<TSavegame>
where
	TSavegame: ThreadSafe + HandlesSaving,
{
	pub fn from_plugin(_: &TSavegame) -> Self {
		Self(PhantomData)
	}
}

impl<TDependencies> RegisterAnimations for AnimationsPlugin<TDependencies> {
	fn register_animations<TAgent>(app: &mut App)
	where
		TAgent: Component + GetAnimationDefinitions + ConfigureNewAnimationDispatch,
		for<'a> AnimationMask: From<&'a TAgent::TAnimationMask>,
		for<'a> AffectedAnimationBones: From<&'a TAgent::TAnimationMask>,
	{
		app.register_derived_component::<TAgent, AnimationDispatch>()
			.add_systems(
				Update,
				(
					TAgent::init_animation_components::<AnimationGraph, AssetServer>,
					TAgent::mask_animation_nodes,
					TAgent::set_animation_mask_bones,
					TAgent::remove_unused_animation_targets,
				)
					.chain()
					.in_set(AnimationSystems),
			);
	}

	fn register_movement_direction<TMovementDirection>(app: &mut App)
	where
		TMovementDirection: Component + GetMovementDirection,
	{
		app.add_systems(
			Update,
			AnimationDispatch::set_directional_animation_weights::<TMovementDirection>
				.in_set(AnimationSystems),
		);
	}
}

impl<TDependencies> HasAnimationsDispatch for AnimationsPlugin<TDependencies> {
	type TAnimationDispatch = AnimationDispatch;
}

impl<TSavegame> Plugin for AnimationsPlugin<TSavegame>
where
	TSavegame: ThreadSafe + HandlesSaving,
{
	fn build(&self, app: &mut App) {
		TSavegame::register_savable_component::<AnimationDispatch>(app);

		app.add_systems(
			Update,
			(
				AnimationDispatch::play_animation_clip_via::<&mut AnimationPlayer>,
				AnimationDispatch::track_in_self_and_children::<AnimationPlayer>().system(),
				AnimationDispatch::track_in_self_and_children::<AnimationGraphHandle>().system(),
				AnimationDispatch::init_player_components::<AnimationGraphHandle>,
			)
				.in_set(AnimationSystems),
		);
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct AnimationSystems;

impl<TDependencies> SystemSetDefinition for AnimationsPlugin<TDependencies> {
	type TSystemSet = AnimationSystems;

	const SYSTEMS: Self::TSystemSet = AnimationSystems;
}
