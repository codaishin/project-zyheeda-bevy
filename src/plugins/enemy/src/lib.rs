pub mod components;
mod systems;

use bevy::prelude::*;
use common::{
	attributes::{affected_by::AffectedBy, health::Health},
	effects::{deal_damage::DealDamage, gravity::Gravity},
	traits::{
		handles_effect::HandlesEffect,
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
	},
};
use components::void_sphere::VoidSphere;
use std::marker::PhantomData;
use systems::{base_behavior::base_enemy_behavior, void_sphere::ring_rotation::ring_rotation};

pub struct EnemyPlugin<TPrefabsPlugin, TInteractionsPlugin>(
	PhantomData<(TPrefabsPlugin, TInteractionsPlugin)>,
);

impl<TPrefabsPlugin, TInteractionsPlugin> EnemyPlugin<TPrefabsPlugin, TInteractionsPlugin> {
	pub fn depends_on(_: &TPrefabsPlugin, _: &TInteractionsPlugin) -> Self {
		Self(PhantomData::<(TPrefabsPlugin, TInteractionsPlugin)>)
	}
}

impl<TPrefabsPlugin, TInteractionsPlugin> Plugin
	for EnemyPlugin<TPrefabsPlugin, TInteractionsPlugin>
where
	TPrefabsPlugin: Plugin + RegisterPrefab,
	TInteractionsPlugin: Plugin
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>,
{
	fn build(&self, app: &mut App) {
		TPrefabsPlugin::with_dependency::<TInteractionsPlugin>().register_prefab::<VoidSphere>(app);

		app.add_systems(Update, ring_rotation)
			.add_systems(Update, base_enemy_behavior);
	}
}
