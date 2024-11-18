pub mod bundle;
pub mod components;

mod animation_keys;
mod systems;

use animations::animation::Animation;
use bevy::prelude::*;
use common::traits::animation::{RegisterAnimations, StartAnimation};
use components::player::Player;
use prefabs::traits::RegisterPrefab;
use std::marker::PhantomData;
use systems::{move_player::move_player, toggle_walk_run::player_toggle_walk_run};

pub struct PlayerPlugin<TAnimationPlugin, TAnimationDispatch>(
	PhantomData<(TAnimationPlugin, TAnimationDispatch)>,
);

impl<TAnimationPlugin, TAnimationDispatch> PlayerPlugin<TAnimationPlugin, TAnimationDispatch>
where
	TAnimationDispatch: Component + StartAnimation<Animation> + Default,
{
	pub fn new(animation_plugin: &mut TAnimationPlugin) -> Self
	where
		TAnimationPlugin: RegisterAnimations<TAnimationDispatch>,
	{
		animation_plugin.register_animations::<Player>();
		Self(PhantomData)
	}
}

impl<TAnimationPlugin, TAnimationDispatch> Plugin
	for PlayerPlugin<TAnimationPlugin, TAnimationDispatch>
where
	TAnimationPlugin: Sync + Send + 'static,
	TAnimationDispatch: Component,
{
	fn build(&self, app: &mut App) {
		app.register_prefab::<Player>()
			.add_systems(Update, (player_toggle_walk_run, move_player));
	}
}
