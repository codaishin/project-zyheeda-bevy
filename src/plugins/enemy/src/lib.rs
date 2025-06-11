pub mod components;

mod systems;
mod traits;

use bevy::prelude::*;
use common::{
	attributes::{affected_by::AffectedBy, health::Health},
	effects::{deal_damage::DealDamage, force::Force, gravity::Gravity},
	traits::{
		handles_effect::HandlesEffect,
		handles_enemies::HandlesEnemies,
		handles_game_states::HandlesGameStates,
		handles_interactions::HandlesInteractions,
		handles_saving::HandlesSaving,
		prefab::AddPrefabObserver,
		thread_safe::ThreadSafe,
	},
};
use components::{enemy::Enemy, void_beam::VoidBeam, void_sphere::VoidSphere};
use std::marker::PhantomData;
use systems::void_sphere::ring_rotation::ring_rotation;

pub struct EnemyPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TGameStates, TSaveGame, TInteractions> EnemyPlugin<(TGameStates, TSaveGame, TInteractions)>
where
	TGameStates: ThreadSafe + HandlesGameStates,
	TSaveGame: ThreadSafe + HandlesSaving,
	TInteractions: ThreadSafe
		+ HandlesInteractions
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
{
	pub fn from_plugins(_: &TGameStates, _: &TSaveGame, _: &TInteractions) -> Self {
		Self(PhantomData)
	}
}

impl<TGameStates, TSaveGame, TInteractions> Plugin
	for EnemyPlugin<(TGameStates, TSaveGame, TInteractions)>
where
	TGameStates: ThreadSafe + HandlesGameStates,
	TSaveGame: ThreadSafe + HandlesSaving,
	TInteractions: ThreadSafe
		+ HandlesInteractions
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>
		+ HandlesEffect<Force, TTarget = AffectedBy<Force>>,
{
	fn build(&self, app: &mut App) {
		TGameStates::on_starting_new_game(app, VoidSphere::spawn);

		app.register_required_components::<Enemy, TSaveGame::TSaveEntityMarker>()
			.add_prefab_observer::<VoidSphere, TInteractions>()
			.add_prefab_observer::<VoidBeam, TInteractions>()
			.add_systems(Update, ring_rotation);
	}
}

impl<TDependencies> HandlesEnemies for EnemyPlugin<TDependencies> {
	type TEnemy = Enemy;
}
