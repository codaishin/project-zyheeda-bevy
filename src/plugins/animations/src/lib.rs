mod components;
mod system_params;
mod systems;
mod traits;

#[cfg(test)]
pub(crate) mod test_tools;

use crate::{
	components::animation_lookup::{AnimationClips, AnimationLookup2},
	system_params::animations::{AnimationsParamMut, override_animations::AnimationOverrideEvent},
	systems::{
		discover_animation_mask_bones::DiscoverMaskChains,
		init_animation_components::InitAnimationComponents,
		init_animation_mask::MaskAllBits,
		mask_animation_nodes::MaskAnimationNodes,
		play_animation_clip2::PlayAnimationClip2,
		remove_unused_animation_targets::RemoveUnusedAnimationTargets,
		set_directional_animation_weights::SetDirectionalAnimationWeights,
	},
};
use bevy::prelude::*;
use common::{
	systems::track_components::TrackComponentInSelfAndChildren,
	traits::{
		animation::{
			AffectedAnimationBones,
			Animation,
			AnimationKey,
			ConfigureNewAnimationDispatch,
			GetAnimationDefinitions,
			GetMovementDirection,
			HandlesAnimations,
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
	dispatch_player_components::DispatchPlayerComponents,
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
		type DispatchNew = AnimationDispatch<AnimationKey>;
		TSavegame::register_savable_component::<DispatchNew>(app);
		app.add_observer(AnimationOverrideEvent::observe);
		app.add_systems(
			Update,
			(
				AnimationGraphHandle::init_animation_mask::<AnimationClips>,
				AnimationLookup2::init_animation_bone_groups,
				DispatchNew::track_in_self_and_children::<AnimationPlayer>().system(),
				DispatchNew::track_in_self_and_children::<AnimationGraphHandle>().system(),
				DispatchNew::distribute_player_components::<AnimationGraphHandle>,
				DispatchNew::play_animation_clip_via2::<&mut AnimationPlayer>,
			)
				.chain()
				.in_set(AnimationSystems),
		);

		// FIXME: Remove when all consumers use new `HandlesAnimations` interface
		type DispatchOld = AnimationDispatch<Animation>;
		TSavegame::register_savable_component::<DispatchOld>(app);
		app.add_systems(
			Update,
			(
				DispatchOld::play_animation_clip_via::<&mut AnimationPlayer>,
				DispatchOld::track_in_self_and_children::<AnimationPlayer>().system(),
				DispatchOld::track_in_self_and_children::<AnimationGraphHandle>().system(),
				DispatchOld::distribute_player_components::<AnimationGraphHandle>,
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

impl<TDependencies> HandlesAnimations for AnimationsPlugin<TDependencies> {
	type TAnimationsMut<'w, 's> = AnimationsParamMut<'w, 's>;
}
