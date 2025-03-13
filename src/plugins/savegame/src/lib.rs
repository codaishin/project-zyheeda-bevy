pub mod components;

mod context;
mod traits;

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use common::states::game_state::GameState;
use components::save::Save;
use context::SaveContext;

pub struct SavegamePlugin;

impl Plugin for SavegamePlugin {
	fn build(&self, app: &mut App) {
		let context = Arc::new(Mutex::new(SaveContext));

		app.add_systems(OnEnter(GameState::Saving), Save::save_system_via(context));
	}
}
