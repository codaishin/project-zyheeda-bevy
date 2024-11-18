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
		init_associated_component::{GetAssociated, InitAssociatedComponent},
		track_components::TrackComponentInSelfAndChildren,
	},
	traits::animation::{GetAnimationPaths, RegisterAnimations},
};
use components::animation_dispatch::AnimationDispatch;
use resource::AnimationData;
use std::collections::HashSet;
use systems::play_animation_clip::PlayAnimationClip;

#[derive(Default)]
pub struct AnimationsPlugin {
	setups: HashSet<fn(&mut App)>,
}

impl RegisterAnimations<AnimationDispatch> for AnimationsPlugin {
	fn register_animations<
		TAgent: Component + GetAnimationPaths + GetAssociated<AnimationDispatch>,
	>(
		&mut self,
	) {
		self.setups.insert(|app| {
			app.add_systems(
				Startup,
				TAgent::init_animation_clips::<AnimationGraph, AssetServer>,
			)
			.add_systems(
				Update,
				(
					TAgent::init_associated::<AnimationDispatch>,
					TAgent::init_animation_graph_and_transitions::<AnimationDispatch>,
				),
			);
		});
	}
}

impl Plugin for AnimationsPlugin {
	fn build(&self, app: &mut App) {
		type AnimationQuery<'a> = (Mut<'a, AnimationPlayer>, Mut<'a, AnimationTransitions>);

		for setup in &self.setups {
			setup(app);
		}

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
