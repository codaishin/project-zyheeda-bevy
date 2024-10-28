mod systems;

use bevy::prelude::*;
use systems::toggle_walk_run::player_toggle_walk_run;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, player_toggle_walk_run);
	}
}
