pub mod bundle;
pub mod components;

mod systems;

use bevy::prelude::*;
use common::traits::{animation::RegisterAnimations, prefab::RegisterPrefab};
use components::player::Player;
use std::marker::PhantomData;
use systems::{move_player::move_player, toggle_walk_run::player_toggle_walk_run};

pub struct PlayerPlugin<TAnimationPlugin, TPrefabsPlugin>(
	PhantomData<(TAnimationPlugin, TPrefabsPlugin)>,
);

impl<TAnimationPlugin, TPrefabsPlugin> PlayerPlugin<TAnimationPlugin, TPrefabsPlugin>
where
	TAnimationPlugin: RegisterAnimations,
{
	pub fn depends_on(_: &TAnimationPlugin, _: &TPrefabsPlugin) -> Self {
		Self(PhantomData::<(TAnimationPlugin, TPrefabsPlugin)>)
	}
}

impl<TAnimationPlugin, TPrefabsPlugin> Plugin for PlayerPlugin<TAnimationPlugin, TPrefabsPlugin>
where
	TAnimationPlugin: Plugin + RegisterAnimations,
	TPrefabsPlugin: Plugin + RegisterPrefab,
{
	fn build(&self, app: &mut App) {
		TAnimationPlugin::register_animations::<Player>(app);
		TPrefabsPlugin::register_prefab::<Player>(app);

		app.add_systems(Update, (player_toggle_walk_run, move_player));
	}
}
