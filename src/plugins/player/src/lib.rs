pub mod components;

mod animation_keys;
mod systems;

use animations::traits::RegisterAnimations;
use bevy::prelude::*;
use components::player::Player;
use systems::{move_player::move_player, toggle_walk_run::player_toggle_walk_run};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.register_animations::<Player>()
			.add_systems(Update, (player_toggle_walk_run, move_player));
	}
}
