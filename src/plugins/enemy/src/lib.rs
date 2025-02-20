pub mod components;

mod systems;
mod traits;

use bevy::prelude::*;
use common::{
	attributes::{affected_by::AffectedBy, health::Health},
	components::insert_asset::InsertAssetFromSource,
	effects::{deal_damage::DealDamage, gravity::Gravity},
	traits::{
		handles_effect::HandlesEffect,
		handles_enemies::HandlesEnemies,
		handles_game_states::HandlesGameStates,
		handles_interactions::HandlesInteractions,
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
		thread_safe::ThreadSafe,
	},
};
use components::{
	enemy::Enemy,
	void_beam::{VoidBeam, VoidBeamModel},
	void_sphere::VoidSphere,
};
use std::marker::PhantomData;
use systems::void_sphere::ring_rotation::ring_rotation;

pub struct EnemyPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TGameStates, TPrefabs, TInteractions> EnemyPlugin<(TGameStates, TPrefabs, TInteractions)>
where
	TGameStates: ThreadSafe + HandlesGameStates,
	TPrefabs: ThreadSafe + RegisterPrefab,
	TInteractions: ThreadSafe
		+ HandlesInteractions
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
{
	pub fn depends_on(_: &TGameStates, _: &TPrefabs, _: &TInteractions) -> Self {
		Self(PhantomData)
	}
}

impl<TGameStates, TPrefabs, TInteractions> Plugin
	for EnemyPlugin<(TGameStates, TPrefabs, TInteractions)>
where
	TGameStates: ThreadSafe + HandlesGameStates,
	TPrefabs: ThreadSafe + RegisterPrefab,
	TInteractions: ThreadSafe
		+ HandlesInteractions
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
{
	fn build(&self, app: &mut App) {
		TGameStates::on_starting_new_game(app, VoidSphere::spawn);
		TPrefabs::with_dependency::<TInteractions>().register_prefab::<VoidSphere>(app);
		TPrefabs::with_dependency::<TInteractions>().register_prefab::<VoidBeam>(app);

		app.add_systems(
			Update,
			InsertAssetFromSource::<StandardMaterial, VoidBeamModel>::system,
		)
		.add_systems(Update, ring_rotation);
	}
}

impl<TDependencies> HandlesEnemies for EnemyPlugin<TDependencies> {
	type TEnemy = Enemy;
}
