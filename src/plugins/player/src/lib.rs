pub mod bundle;
pub mod components;

mod systems;

use bevy::prelude::*;
use common::{
	attributes::health::Health,
	effects::deal_damage::DealDamage,
	traits::{
		animation::RegisterAnimations,
		handles_effect::HandlesEffect,
		handles_lights::HandlesLights,
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
	},
};
use components::player::Player;
use std::marker::PhantomData;
use systems::{move_player::move_player, toggle_walk_run::player_toggle_walk_run};

pub struct PlayerPlugin<TAnimation, TPrefabs, TInteractions, TLights>(
	PhantomData<(TAnimation, TPrefabs, TInteractions, TLights)>,
);

impl<TAnimation, TPrefabs, TInteractions, TLights>
	PlayerPlugin<TAnimation, TPrefabs, TInteractions, TLights>
{
	pub fn depends_on(_: &TAnimation, _: &TPrefabs, _: &TInteractions, _: &TLights) -> Self {
		Self(PhantomData::<(TAnimation, TPrefabs, TInteractions, TLights)>)
	}
}

impl<TAnimation, TPrefabs, TInteractions, TLights> Plugin
	for PlayerPlugin<TAnimation, TPrefabs, TInteractions, TLights>
where
	TAnimation: Plugin + RegisterAnimations,
	TPrefabs: Plugin + RegisterPrefab,
	TInteractions: Plugin + HandlesEffect<DealDamage, TTarget = Health>,
	TLights: Plugin + HandlesLights,
{
	fn build(&self, app: &mut App) {
		TAnimation::register_animations::<Player>(app);
		TPrefabs::with_dependency::<(TInteractions, TLights)>().register_prefab::<Player>(app);

		app.add_systems(Update, (player_toggle_walk_run, move_player));
	}
}
