pub mod bundle;
pub mod components;

use bevy::prelude::*;
use common::{
	attributes::health::Health,
	effects::deal_damage::DealDamage,
	traits::{
		animation::RegisterAnimations,
		handles_bars::HandlesBars,
		handles_behaviors::HandlesBehaviors,
		handles_effect::HandlesEffect,
		handles_lights::HandlesLights,
		handles_player::HandlesPlayer,
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
	},
};
use components::{player::Player, player_movement::PlayerMovement};
use std::marker::PhantomData;

pub struct PlayerPlugin<TAnimation, TPrefabs, TInteractions, TLights, TBars, TBehaviors>(
	PhantomData<(
		TAnimation,
		TPrefabs,
		TInteractions,
		TLights,
		TBars,
		TBehaviors,
	)>,
);

impl<TAnimation, TPrefabs, TInteractions, TLights, TBars, TBehaviors>
	PlayerPlugin<TAnimation, TPrefabs, TInteractions, TLights, TBars, TBehaviors>
{
	pub fn depends_on(
		_: &TAnimation,
		_: &TPrefabs,
		_: &TInteractions,
		_: &TLights,
		_: &TBars,
		_: &TBehaviors,
	) -> Self {
		Self(
			PhantomData::<(
				TAnimation,
				TPrefabs,
				TInteractions,
				TLights,
				TBars,
				TBehaviors,
			)>,
		)
	}
}

impl<TAnimation, TPrefabs, TInteractions, TLights, TBars, TBehaviors> Plugin
	for PlayerPlugin<TAnimation, TPrefabs, TInteractions, TLights, TBars, TBehaviors>
where
	TAnimation: Plugin + RegisterAnimations,
	TPrefabs: Plugin + RegisterPrefab,
	TInteractions: Plugin + HandlesEffect<DealDamage, TTarget = Health>,
	TLights: Plugin + HandlesLights,
	TBars: Plugin + HandlesBars,
	TBehaviors: Plugin + HandlesBehaviors,
{
	fn build(&self, app: &mut App) {
		TAnimation::register_animations::<Player>(app);
		TBehaviors::register_camera_orbit_for::<Player>(app);
		TBehaviors::register_player_movement::<PlayerMovement>(app);
		TPrefabs::with_dependency::<(TInteractions, TLights, TBars)>()
			.register_prefab::<Player>(app);
	}
}

impl<TAnimation, TPrefabs, TInteractions, TLights, TBars, TBehaviors> HandlesPlayer
	for PlayerPlugin<TAnimation, TPrefabs, TInteractions, TLights, TBars, TBehaviors>
{
	type TPlayer = Player;
}
