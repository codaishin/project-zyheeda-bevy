mod components;
mod systems;

use crate::components::enemy::Enemy;
use bevy::prelude::*;
use common::traits::{
	handles_enemies::HandlesEnemies,
	handles_physics::{HandlesAllPhysicalEffects, HandlesPhysicalObjects},
	handles_saving::HandlesSaving,
	prefab::AddPrefabObserver,
	thread_safe::ThreadSafe,
};
use std::marker::PhantomData;
use systems::void_sphere::ring_rotation::ring_rotation;

pub struct EnemyPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSaveGame, TPhysics> EnemyPlugin<(TSaveGame, TPhysics)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesPhysicalObjects + HandlesAllPhysicalEffects,
{
	pub fn from_plugins(_: &TSaveGame, _: &TPhysics) -> Self {
		Self(PhantomData)
	}
}

impl<TSaveGame, TPhysics> Plugin for EnemyPlugin<(TSaveGame, TPhysics)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesPhysicalObjects + HandlesAllPhysicalEffects,
{
	fn build(&self, app: &mut App) {
		// Save config
		TSaveGame::register_savable_component::<Enemy>(app);
		app.register_required_components::<Enemy, TSaveGame::TSaveEntityMarker>();

		// prefabs
		app.add_prefab_observer::<Enemy, TPhysics>();

		// behaviors
		app.add_systems(Update, ring_rotation);
	}
}

impl<TDependencies> HandlesEnemies for EnemyPlugin<TDependencies> {
	type TEnemy = Enemy;
}
