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
		handles_interactions::HandlesInteractions,
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
	},
};
use components::{enemy::Enemy, void_beam::VoidBeam, void_sphere::VoidSphere};
use std::marker::PhantomData;
use systems::void_sphere::ring_rotation::ring_rotation;

pub struct EnemyPlugin<TPrefabs, TInteractions, TBars>(
	PhantomData<(TPrefabs, TInteractions, TBars)>,
);

impl<TPrefabs, TInteractions, TBars> EnemyPlugin<TPrefabs, TInteractions, TBars> {
	pub fn depends_on(_: &TPrefabs, _: &TInteractions, _: &TBars) -> Self {
		Self(PhantomData::<(TPrefabs, TInteractions, TBars)>)
	}
}

impl<TPrefabs, TInteractions, TBars> Plugin for EnemyPlugin<TPrefabs, TInteractions, TBars>
where
	TPrefabs: Plugin + RegisterPrefab,
	TInteractions: Plugin
		+ HandlesInteractions
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
	TBars: Plugin + HandlesBars,
{
	fn build(&self, app: &mut App) {
		TPrefabs::with_dependency::<(TInteractions, TBars)>().register_prefab::<VoidSphere>(app);
		TPrefabs::with_dependency::<TInteractions>().register_prefab::<VoidBeam>(app);

		app.add_systems(Update, ring_rotation);
	}
}

impl<TPrefabs, TInteractions, TBars> HandlesEnemies
	for EnemyPlugin<TPrefabs, TInteractions, TBars>
{
	type TEnemy = Enemy;
}
