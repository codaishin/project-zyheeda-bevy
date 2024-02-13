mod systems;

use bevy::app::{App, Plugin, PostUpdate};
use systems::collision::destroy_on_collision::destroy_on_collision;

pub struct InteractionsPlugin;

impl Plugin for InteractionsPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PostUpdate, destroy_on_collision);
	}
}
