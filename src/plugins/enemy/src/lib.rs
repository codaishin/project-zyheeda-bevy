pub mod components;

mod systems;

use bevy::prelude::*;
use common::{
	attributes::{affected_by::AffectedBy, health::Health},
	effects::{deal_damage::DealDamage, gravity::Gravity},
	traits::{
		handles_bars::HandlesBars,
		handles_behaviors::HandlesBehaviors,
		handles_effect::HandlesEffect,
		handles_player::HandlesPlayer,
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
	},
};
use components::{enemy::Enemy, void_sphere::VoidSphere};
use std::marker::PhantomData;
use systems::void_sphere::ring_rotation::ring_rotation;

pub struct EnemyPlugin<TPrefabs, TInteractions, TBars, TPlayers, TBehaviors>(
	PhantomData<(TPrefabs, TInteractions, TBars, TPlayers, TBehaviors)>,
);

impl<TPrefabs, TInteractions, TBars, TPlayers, TBehaviors>
	EnemyPlugin<TPrefabs, TInteractions, TBars, TPlayers, TBehaviors>
{
	pub fn depends_on(
		_: &TPrefabs,
		_: &TInteractions,
		_: &TBars,
		_: &TPlayers,
		_: &TBehaviors,
	) -> Self {
		Self(PhantomData::<(TPrefabs, TInteractions, TBars, TPlayers, TBehaviors)>)
	}
}

impl<TPrefabs, TInteractions, TBars, TPlayers, TBehaviors> Plugin
	for EnemyPlugin<TPrefabs, TInteractions, TBars, TPlayers, TBehaviors>
where
	TPrefabs: Plugin + RegisterPrefab,
	TInteractions: Plugin
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
	TBars: Plugin + HandlesBars,
	TPlayers: Plugin + HandlesPlayer,
	TBehaviors: Plugin + HandlesBehaviors,
{
	fn build(&self, app: &mut App) {
		TBehaviors::register_enemies_for::<TPlayers::TPlayer, Enemy>(app);
		TPrefabs::with_dependency::<(TInteractions, TBars)>().register_prefab::<VoidSphere>(app);

		app.add_systems(Update, ring_rotation);
	}
}
