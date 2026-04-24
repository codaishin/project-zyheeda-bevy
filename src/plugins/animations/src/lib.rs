mod components;
mod observers;
mod system_params;
mod systems;
mod traits;

#[cfg(test)]
pub(crate) mod test_tools;

use crate::{
	components::{
		animation_dispatch::{AnimationGraphOf, AnimationPlayerOf},
		animation_lookup::AnimationClips,
		setup_animations::SetupAnimations,
	},
	system_params::animations::AnimationsParamMut,
	systems::{
		play_animation_clip::PlayAnimationClip,
		set_directional_animation_weights::SetDirectionalAnimationWeights,
		set_pitch_animation_weights::SetPitchAnimationWeights,
	},
};
use bevy::{prelude::*, scene::SceneInstanceReady};
use common::{
	systems::link_children::LinkDescendants,
	tools::plugin_system_set::PluginSystemSet,
	traits::{
		handles_animations::HandlesAnimations,
		handles_saving::HandlesSaving,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::animation_dispatch::AnimationDispatch;
use std::marker::PhantomData;

pub struct AnimationsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSavegame> AnimationsPlugin<TSavegame>
where
	TSavegame: ThreadSafe + HandlesSaving,
{
	pub fn from_plugin(_: &TSavegame) -> Self {
		Self(PhantomData)
	}
}

impl<TSavegame> Plugin for AnimationsPlugin<TSavegame>
where
	TSavegame: ThreadSafe + HandlesSaving,
{
	fn build(&self, app: &mut App) {
		TSavegame::register_savable_component::<AnimationDispatch>(app);
		app.add_observer(SetupAnimations::insert_when::<SceneInstanceReady>);
		app.add_systems(
			Update,
			(
				SetupAnimations::init_masks::<AnimationGraphHandle, AnimationClips>,
				SetupAnimations::init_bone_groups::<AnimationGraphHandle>,
				SetupAnimations::remove_unused_animation_targets::<AnimationGraphHandle>,
				SetupAnimations::stop,
				AnimationDispatch::link_descendants::<AnimationPlayerOf, Added<AnimationPlayer>>,
				AnimationDispatch::link_descendants::<AnimationGraphOf, Added<AnimationGraphHandle>>,
				AnimationDispatch::distribute_player_components,
				AnimationDispatch::play_animation_clip::<&mut AnimationPlayer>,
				AnimationDispatch::set_directional_animation_weights,
				AnimationDispatch::set_pitch_animation_weights,
			)
				.chain()
				.in_set(AnimationSystems),
		);
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct AnimationSystems;

impl<TDependencies> SystemSetDefinition for AnimationsPlugin<TDependencies> {
	type TSystemSet = AnimationSystems;

	const SYSTEMS: PluginSystemSet<Self::TSystemSet> = PluginSystemSet::from_set(AnimationSystems);
}

impl<TDependencies> HandlesAnimations for AnimationsPlugin<TDependencies> {
	type TAnimationsMut<'w, 's> = AnimationsParamMut<'w, 's>;
}
