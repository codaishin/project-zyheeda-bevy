pub mod components;

mod systems;

use bevy::prelude::*;
use common::{
	attributes::health::Health,
	effects::deal_damage::DealDamage,
	traits::{
		animation::RegisterAnimations,
		handles_bars::HandlesBars,
		handles_effect::HandlesEffect,
		handles_game_states::HandlesGameStates,
		handles_lights::HandlesLights,
		handles_player::{ConfiguresPlayerMovement, HandlesPlayer},
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
	},
};
use components::{player::Player, player_movement::PlayerMovement};
use std::marker::PhantomData;
use systems::toggle_walk_run::player_toggle_walk_run;

pub struct PlayerPlugin<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>(
	PhantomData<(
		TGameStates,
		TAnimation,
		TPrefabs,
		TInteractions,
		TLights,
		TBars,
	)>,
);

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
	PlayerPlugin<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	pub fn depends_on(
		_: &TGameStates,
		_: &TAnimation,
		_: &TPrefabs,
		_: &TInteractions,
		_: &TLights,
		_: &TBars,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> Plugin
	for PlayerPlugin<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
where
	TGameStates: Plugin + HandlesGameStates,
	TAnimation: Plugin + RegisterAnimations,
	TPrefabs: Plugin + RegisterPrefab,
	TInteractions: Plugin + HandlesEffect<DealDamage, TTarget = Health>,
	TLights: Plugin + HandlesLights,
	TBars: Plugin + HandlesBars,
{
	fn build(&self, app: &mut App) {
		TGameStates::on_starting_new_game(app, Player::spawn);
		TAnimation::register_animations::<Player>(app);
		TPrefabs::with_dependency::<(TInteractions, TLights, TBars)>()
			.register_prefab::<Player>(app);

		app.add_systems(Update, player_toggle_walk_run);
	}
}

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> HandlesPlayer
	for PlayerPlugin<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TPlayer = Player;
}

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> ConfiguresPlayerMovement
	for PlayerPlugin<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TPlayerMovement = PlayerMovement;
}
