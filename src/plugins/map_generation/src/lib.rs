mod components;
mod systems;
mod traits;

use bevy::app::{App, Plugin, Update};
use components::{Corner, Wall};
use systems::add_colliders::add_colliders;

pub struct MapGenerationPlugin;

impl Plugin for MapGenerationPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, (add_colliders::<Wall>, add_colliders::<Corner>));
	}
}
