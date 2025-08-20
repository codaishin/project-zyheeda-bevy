pub mod components;

mod systems;

use crate::components::enemy::Enemy;
use bevy::prelude::*;
use common::{
	attributes::{affected_by::AffectedBy, health::Health},
	effects::{deal_damage::DealDamage, force::Force, gravity::Gravity},
	traits::{
		handles_effect::HandlesEffect,
		handles_enemies::{HandlesEnemies, HandlesEnemyConfig},
		handles_interactions::HandlesInteractions,
		handles_saving::HandlesSaving,
		prefab::AddPrefabObserver,
		thread_safe::ThreadSafe,
	},
};
use components::void_sphere::VoidSphere;
use std::marker::PhantomData;
use systems::void_sphere::ring_rotation::ring_rotation;

pub struct EnemyPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSaveGame, TInteractions> EnemyPlugin<(TSaveGame, TInteractions)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TInteractions: ThreadSafe
		+ HandlesInteractions
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
{
	pub fn from_plugins(_: &TSaveGame, _: &TInteractions) -> Self {
		Self(PhantomData)
	}
}

impl<TSaveGame, TInteractions> Plugin for EnemyPlugin<(TSaveGame, TInteractions)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TInteractions: ThreadSafe
		+ HandlesInteractions
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>
		+ HandlesEffect<Force, TTarget = AffectedBy<Force>>,
{
	fn build(&self, app: &mut App) {
		// Save config
		TSaveGame::register_savable_component::<VoidSphere>(app);
		app.register_required_components::<VoidSphere, TSaveGame::TSaveEntityMarker>();

		// prefabs
		app.add_prefab_observer::<VoidSphere, TInteractions>();

		// behaviors
		app.add_systems(Update, ring_rotation);
	}
}

impl<TDependencies> HandlesEnemies for EnemyPlugin<TDependencies> {
	type TEnemy = VoidSphere;
}

impl<TDependencies> HandlesEnemyConfig for EnemyPlugin<TDependencies> {
	type TEnemyBehavior = Enemy;
}
