mod components;
mod observers;
mod system_params;
mod systems;
mod traits;

#[cfg(test)]
pub(crate) mod test_tools;

use crate::{
	components::{animation_lookup::AnimationClips, setup_animations::SetupAnimations},
	system_params::animations::AnimationsParamMut,
	systems::{
		play_animation_clip::PlayAnimationClip2,
		set_directional_animation_weights::SetDirectionalAnimationWeights,
	},
};
use bevy::{prelude::*, scene::SceneInstanceReady};
use common::{
	systems::track_components::TrackComponentInSelfAndChildren,
	traits::{
		animation::HandlesAnimations,
		handles_saving::HandlesSaving,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::animation_dispatch::AnimationDispatch;
use std::marker::PhantomData;
use systems::dispatch_player_components::DispatchPlayerComponents;

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
				AnimationDispatch::track_in_self_and_children::<AnimationPlayer>().system(),
				AnimationDispatch::track_in_self_and_children::<AnimationGraphHandle>().system(),
				AnimationDispatch::distribute_player_components::<AnimationGraphHandle>,
				AnimationDispatch::play_animation_clip::<&mut AnimationPlayer>,
				AnimationDispatch::set_directional_animation_weights,
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

	const SYSTEMS: Self::TSystemSet = AnimationSystems;
}

impl<TDependencies> HandlesAnimations for AnimationsPlugin<TDependencies> {
	type TAnimationsMut<'w, 's> = AnimationsParamMut<'w, 's>;
}
