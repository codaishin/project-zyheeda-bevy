pub mod components;

mod systems;
mod traits;

use bevy::prelude::*;
use common::{
	attributes::{affected_by::AffectedBy, health::Health},
	effects::{deal_damage::DealDamage, gravity::Gravity},
	traits::{
		handles_effect::HandlesEffect,
		handles_enemies::HandlesEnemies,
		handles_game_states::HandlesGameStates,
		handles_interactions::HandlesInteractions,
		prefab::AddPrefabObserver,
		thread_safe::ThreadSafe,
	},
};
use components::{enemy::Enemy, void_beam::VoidBeam, void_sphere::VoidSphere};
use std::marker::PhantomData;
use systems::void_sphere::ring_rotation::ring_rotation;

pub struct EnemyPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TGameStates, TInteractions> EnemyPlugin<(TGameStates, TInteractions)>
where
	TGameStates: ThreadSafe + HandlesGameStates,
	TInteractions: ThreadSafe
		+ HandlesInteractions
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
{
	pub fn from_plugins(_: &TGameStates, _: &TInteractions) -> Self {
		Self(PhantomData)
	}
}

impl<TGameStates, TInteractions> Plugin for EnemyPlugin<(TGameStates, TInteractions)>
where
	TGameStates: ThreadSafe + HandlesGameStates,
	TInteractions: ThreadSafe
		+ HandlesInteractions
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
{
	fn build(&self, app: &mut App) {
		TGameStates::on_starting_new_game(app, VoidSphere::spawn);

		app.add_prefab_observer::<VoidSphere, TInteractions>()
			.add_prefab_observer::<VoidBeam, TInteractions>()
			.add_systems(Update, ring_rotation);
	}
}

impl<TDependencies> HandlesEnemies for EnemyPlugin<TDependencies> {
	type TEnemy = Enemy;
}
