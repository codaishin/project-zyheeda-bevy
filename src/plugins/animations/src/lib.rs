pub mod animation;
pub mod components;
pub mod traits;

mod animation_keys;
mod resource;
mod systems;

use animation::Animation;
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
use systems::{
	flush::flush,
	init_animation_clips::init_animation_clips,
	init_animation_player_components::init_animation_player_components,
	play_animation_clip::play_animation_clip,
};
use traits::{GetAnimationPaths, RegisterAnimations};

pub struct AnimationsPlugin;

impl RegisterAnimations for App {
	fn register_animations<TAgent>(&mut self) -> &mut Self
	where
		TAgent: Component + GetAnimationPaths + GetAssociated<AnimationDispatch>,
	{
		self.add_systems(
			Startup,
			init_animation_clips::<TAgent, AnimationGraph, AnimationNodeIndex, AssetServer>,
		)
		.add_systems(
			Update,
			(
				TAgent::init_associated::<AnimationDispatch>,
				init_animation_player_components::<TAgent>,
			)
				.chain(),
		)
	}
}

impl Plugin for AnimationsPlugin {
	fn build(&self, app: &mut App) {
		app.register_animations::<Player>()
			.add_systems(
				Update,
				play_animation_clip::<
					AnimationDispatch,
					(Mut<AnimationPlayer>, Mut<AnimationTransitions>),
				>,
			)
			.add_systems(
				PostUpdate,
				AnimationDispatch::<Animation>::track_in_self_and_children::<AnimationPlayer>(),
			)
			.add_systems(PostUpdate, flush::<AnimationDispatch>);
	}
}
