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
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
	},
};
use components::player::Player;
use std::marker::PhantomData;
use systems::{move_player::move_player, toggle_walk_run::player_toggle_walk_run};

pub struct PlayerPlugin<TAnimationPlugin, TPrefabsPlugin, TInteractionsPlugin>(
	PhantomData<(TAnimationPlugin, TPrefabsPlugin, TInteractionsPlugin)>,
);

impl<TAnimationPlugin, TPrefabsPlugin, TInteractionsPlugin>
	PlayerPlugin<TAnimationPlugin, TPrefabsPlugin, TInteractionsPlugin>
where
	TAnimationPlugin: RegisterAnimations,
{
	pub fn depends_on(_: &TAnimationPlugin, _: &TPrefabsPlugin, _: &TInteractionsPlugin) -> Self {
		Self(PhantomData::<(TAnimationPlugin, TPrefabsPlugin, TInteractionsPlugin)>)
	}
}

impl<TAnimationPlugin, TPrefabsPlugin, TInteractionsPlugin> Plugin
	for PlayerPlugin<TAnimationPlugin, TPrefabsPlugin, TInteractionsPlugin>
where
	TAnimationPlugin: Plugin + RegisterAnimations,
	TPrefabsPlugin: Plugin + RegisterPrefab,
	TInteractionsPlugin: Plugin + HandlesEffect<DealDamage, TTarget = Health>,
{
	fn build(&self, app: &mut App) {
		TAnimationPlugin::register_animations::<Player>(app);
		TPrefabsPlugin::with_dependency::<TInteractionsPlugin>().register_prefab::<Player>(app);

		app.add_systems(Update, (player_toggle_walk_run, move_player));
	}
}
