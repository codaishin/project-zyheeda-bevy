pub mod components;
mod systems;

use bevy::prelude::*;
use common::{
	attributes::{affected_by::AffectedBy, health::Health},
	effects::{deal_damage::DealDamage, gravity::Gravity},
	traits::{
		handles_bars::HandlesBars,
		handles_effect::HandlesEffect,
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
	},
};
use components::void_sphere::VoidSphere;
use std::marker::PhantomData;
use systems::{base_behavior::base_enemy_behavior, void_sphere::ring_rotation::ring_rotation};

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
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
	TBars: Plugin + HandlesBars,
{
	fn build(&self, app: &mut App) {
		TPrefabs::with_dependency::<(TInteractions, TBars)>().register_prefab::<VoidSphere>(app);

		app.add_systems(Update, ring_rotation)
			.add_systems(Update, base_enemy_behavior);
	}
}
