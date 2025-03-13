pub mod components;

mod context;
mod errors;
mod systems;
mod traits;
mod writer;

use crate::systems::buffer::BufferSystem;
use bevy::prelude::*;
use common::{states::game_state::GameState, systems::log::log};
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
				SaveContext::buffer_system(context.clone()).pipe(log),
				SaveContext::flush_system(context).pipe(log),
			)
				.chain(),
		);
	}
}
