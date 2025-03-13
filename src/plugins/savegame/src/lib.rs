pub mod components;

mod context;
mod traits;
mod writer;

use bevy::prelude::*;
use common::{states::game_state::GameState, systems::log::log};
use components::save::Save;
use context::SaveContext;
use std::sync::{Arc, Mutex};
use writer::FileWriter;

pub struct SavegamePlugin;

impl Plugin for SavegamePlugin {
	fn build(&self, app: &mut App) {
		let writer = FileWriter {
			destination: "./quick_save.json",
		};
		let context = Arc::new(Mutex::new(SaveContext::new(writer)));

		app.add_systems(
			OnEnter(GameState::Saving),
			(
				Save::save_system_via(context.clone()),
				SaveContext::flush_system(context).pipe(log),
			)
				.chain(),
		);
	}
}
