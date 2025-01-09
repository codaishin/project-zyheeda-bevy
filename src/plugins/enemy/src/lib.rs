pub mod components;

mod systems;
mod traits;

use bevy::prelude::*;
use common::{
	attributes::{affected_by::AffectedBy, health::Health},
	effects::{deal_damage::DealDamage, gravity::Gravity},
	traits::{
		handles_bars::HandlesBars,
		handles_effect::HandlesEffect,
		handles_enemies::HandlesEnemies,
		handles_game_states::HandlesGameStates,
		handles_interactions::HandlesInteractions,
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
		thread_safe::ThreadSafe,
	},
};
use components::{enemy::Enemy, void_beam::VoidBeam, void_sphere::VoidSphere};
use std::marker::PhantomData;
use systems::void_sphere::ring_rotation::ring_rotation;

pub struct EnemyPlugin<TGameStates, TPrefabs, TInteractions, TBars>(
	PhantomData<(TGameStates, TPrefabs, TInteractions, TBars)>,
);

impl<TGameStates, TPrefabs, TInteractions, TBars>
	EnemyPlugin<TGameStates, TPrefabs, TInteractions, TBars>
{
	pub fn depends_on(_: &TGameStates, _: &TPrefabs, _: &TInteractions, _: &TBars) -> Self {
		Self(PhantomData)
	}
}

impl<TGameStates, TPrefabs, TInteractions, TBars> Plugin
	for EnemyPlugin<TGameStates, TPrefabs, TInteractions, TBars>
where
	TGameStates: ThreadSafe + HandlesGameStates,
	TPrefabs: ThreadSafe + RegisterPrefab,
	TInteractions: ThreadSafe
		+ HandlesInteractions
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
	TBars: ThreadSafe + HandlesBars,
{
	fn build(&self, app: &mut App) {
		TGameStates::on_starting_new_game(app, VoidSphere::spawn);
		TPrefabs::with_dependency::<(TInteractions, TBars)>().register_prefab::<VoidSphere>(app);
		TPrefabs::with_dependency::<TInteractions>().register_prefab::<VoidBeam>(app);

		app.add_systems(Update, ring_rotation);
	}
}

impl<TGameStates, TPrefabs, TInteractions, TBars> HandlesEnemies
	for EnemyPlugin<TGameStates, TPrefabs, TInteractions, TBars>
{
	type TEnemy = Enemy;
}
