pub mod animation;
pub mod components;
pub mod traits;

mod animation_keys;
mod resource;
mod systems;

use crate::systems::{
	init_animation_clips::InitAnimationClips,
	init_animation_graph::InitAnimationGraph,
	play_animation_clip::PlayAnimationClip,
};
use bevy::{
	animation::AnimationPlayer,
	app::{App, Plugin, PostUpdate, Startup, Update},
	asset::AssetServer,
	prelude::*,
};
use common::{
	components::Player,
	systems::{
		init_associated_component::{GetAssociated, InitAssociatedComponent},
		track_components::TrackComponentInChildren,
	},
};
use components::animation_dispatch::AnimationDispatch;
use resource::AnimationData;
use systems::flush::flush;
use traits::{GetAnimationPaths, RegisterAnimations};

pub struct AnimationsPlugin;

impl RegisterAnimations for App {
	fn register_animations<TAgent>(&mut self) -> &mut Self
	where
		TAgent: Component + GetAnimationPaths + GetAssociated<AnimationDispatch>,
	{
		self.add_systems(
			Startup,
			TAgent::init_animation_clips::<AnimationGraph, AssetServer>,
		)
		.add_systems(
			Update,
			(
				TAgent::init_associated::<AnimationDispatch>,
				TAgent::init_animation_graph_and_transitions::<AnimationDispatch>,
			),
		)
	}
}

impl Plugin for AnimationsPlugin {
	fn build(&self, app: &mut App) {
		type AnimationQuery<'a> = (Mut<'a, AnimationPlayer>, Mut<'a, AnimationTransitions>);

		app.register_animations::<Player>()
			.add_systems(
				Update,
				AnimationDispatch::play_animation_clip_via::<AnimationQuery>,
			)
			.add_systems(
				PostUpdate,
				(
					AnimationDispatch::track_in_self_and_children::<AnimationPlayer>(),
					AnimationDispatch::track_in_self_and_children::<AnimationTransitions>(),
				),
			)
			.add_systems(PostUpdate, flush::<AnimationDispatch>);
	}
}
