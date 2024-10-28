mod systems;

use bevy::prelude::*;
use systems::void_sphere::ring_rotation::ring_rotation;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Update, ring_rotation);
	}
}
