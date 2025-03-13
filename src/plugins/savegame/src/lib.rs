pub mod components;

mod resources;

use bevy::prelude::*;

pub struct SavegamePlugin;

impl Plugin for SavegamePlugin {
	fn build(&self, _: &mut App) {}
}
