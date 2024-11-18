pub mod bundle;
pub mod components;

mod systems;

use bevy::prelude::*;
use common::traits::animation::RegisterAnimations;
use components::player::Player;
use prefabs::traits::RegisterPrefab;
use std::marker::PhantomData;
use systems::{move_player::move_player, toggle_walk_run::player_toggle_walk_run};

pub struct PlayerPlugin<TAnimationPlugin>(PhantomData<TAnimationPlugin>);

impl<TAnimationPlugin> PlayerPlugin<TAnimationPlugin>
where
	TAnimationPlugin: RegisterAnimations,
{
	pub fn depends_on(_: &TAnimationPlugin) -> Self {
		Self(PhantomData)
	}
}

impl<TAnimationPlugin> Plugin for PlayerPlugin<TAnimationPlugin>
where
	TAnimationPlugin: Plugin + RegisterAnimations,
{
	fn build(&self, app: &mut App) {
		TAnimationPlugin::register_animations::<Player>(app);

		app.register_prefab::<Player>()
			.add_systems(Update, (player_toggle_walk_run, move_player));
	}
}
