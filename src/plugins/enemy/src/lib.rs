pub mod components;
mod systems;

use bevy::prelude::*;
use common::traits::prefab::RegisterPrefab;
use components::void_sphere::VoidSphere;
use std::marker::PhantomData;
use systems::{base_behavior::base_enemy_behavior, void_sphere::ring_rotation::ring_rotation};

pub struct EnemyPlugin<TPrefabsPlugin>(PhantomData<TPrefabsPlugin>);

impl<TPrefabsPlugin> EnemyPlugin<TPrefabsPlugin> {
	pub fn depends_on(_: &TPrefabsPlugin) -> Self {
		Self(PhantomData)
	}
}

impl<TPrefabsPlugin> Plugin for EnemyPlugin<TPrefabsPlugin>
where
	TPrefabsPlugin: Plugin + RegisterPrefab,
{
	fn build(&self, app: &mut App) {
		TPrefabsPlugin::register_prefab::<VoidSphere>(app);

		app.add_systems(Update, ring_rotation)
			.add_systems(Update, base_enemy_behavior);
	}
}
