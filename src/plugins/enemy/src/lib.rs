pub mod components;
mod systems;

use bevy::prelude::*;
use components::void_sphere::VoidSphere;
use prefabs::traits::RegisterPrefab;
use systems::{base_behavior::base_enemy_behavior, void_sphere::ring_rotation::ring_rotation};

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
	fn build(&self, app: &mut App) {
		app.register_prefab::<VoidSphere>()
			.add_systems(Update, ring_rotation)
			.add_systems(Update, base_enemy_behavior);
	}
}
