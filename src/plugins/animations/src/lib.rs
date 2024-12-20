pub mod animation;
pub mod components;
pub mod systems;
pub mod traits;

mod resource;

use crate::systems::{
	init_animation_clips::InitAnimationClips,
	init_animation_graph::InitAnimationGraph,
};
use bevy::prelude::*;
use common::{
	systems::{
		insert_associated::{Configure, InsertAssociated, InsertOn},
		track_components::TrackComponentInSelfAndChildren,
	},
	traits::animation::{
		ConfigureNewAnimationDispatch,
		GetAnimationPaths,
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
		TAgent: Component + GetAnimationPaths + ConfigureNewAnimationDispatch,
	{
		let configure_dispatch = Configure::Apply(TAgent::configure_animation_dispatch);

		app.add_systems(
			Startup,
			TAgent::init_animation_clips::<AnimationGraph, AssetServer>,
		)
		.add_systems(
			Update,
			(
				InsertOn::<TAgent>::associated::<AnimationDispatch>(configure_dispatch),
				TAgent::init_animation_graph_and_transitions::<AnimationDispatch>,
			),
		);
	}
}

impl HasAnimationsDispatch for AnimationsPlugin {
	type TAnimationDispatch = AnimationDispatch;
}

impl Plugin for AnimationsPlugin {
	fn build(&self, app: &mut App) {
		type AnimationQuery<'a> = (Mut<'a, AnimationPlayer>, Mut<'a, AnimationTransitions>);

		app.add_systems(
			Update,
			AnimationDispatch::play_animation_clip_via::<AnimationQuery>,
		)
		.add_systems(
			PostUpdate,
			(
				AnimationDispatch::track_in_self_and_children::<AnimationPlayer>().system(),
				AnimationDispatch::track_in_self_and_children::<AnimationTransitions>().system(),
			),
		);
	}
}
