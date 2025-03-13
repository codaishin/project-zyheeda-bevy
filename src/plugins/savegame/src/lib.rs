pub mod components;

mod context;
mod traits;
mod writer;

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use common::states::game_state::GameState;
use components::save::Save;
use context::SaveContext;
use writer::FileWriter;

pub struct SavegamePlugin;

impl Plugin for SavegamePlugin {
	fn build(&self, app: &mut App) {
		let writer = FileWriter;
		let context = Arc::new(Mutex::new(SaveContext::new(writer)));

		app.add_systems(
			OnEnter(GameState::Saving),
			(
				Save::save_system_via(context.clone()),
				SaveContext::flush_system(context),
			)
				.chain(),
		);
	}
}
