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
	prelude::{
		AnimationGraph,
		AnimationNodeIndex,
		AnimationTransitions,
		Component,
		IntoSystemConfigs,
		Res,
	},
};
use common::{components::Player, systems::track_components::TrackComponentInChildren};
use components::animation_dispatch::AnimationDispatch;
use resource::AnimationData;
use systems::{
	flush::flush,
	init_animation_clips::init_animation_clips,
	init_animation_components::init_animation_components,
	play_animation_clip::play_animation_clip,
};
use traits::{GetAnimationDispatch, GetAnimationPaths, RegisterAnimations};

pub struct AnimationsPlugin;

impl RegisterAnimations for App {
	fn register_animations<TAgent>(&mut self) -> &mut Self
	where
		TAgent: Component + GetAnimationPaths + GetAnimationDispatch,
	{
		let init_animations_clips =
			init_animation_clips::<TAgent, AnimationGraph, AnimationNodeIndex, AssetServer>;
		let init_animation_components = init_animation_components::<TAgent>;
		let play_animation_clip = play_animation_clip::<
			TAgent,
			Animation,
			AnimationDispatch,
			AnimationNodeIndex,
			AnimationPlayer,
			AnimationTransitions,
		>;

		self.add_systems(Startup, init_animations_clips)
			.add_systems(
				Update,
				(init_animation_components, play_animation_clip).chain(),
			)
	}
}

impl Plugin for AnimationsPlugin {
	fn build(&self, app: &mut App) {
		app.register_animations::<Player>()
			.add_systems(
				PostUpdate,
				AnimationDispatch::<Animation>::track_in_self_and_children::<AnimationPlayer>(),
			)
			.add_systems(PostUpdate, flush::<AnimationDispatch>);
	}
}
