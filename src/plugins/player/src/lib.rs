pub mod bundle;
pub mod components;

mod systems;

use bevy::prelude::*;
use common::{
	attributes::health::Health,
	effects::deal_damage::DealDamage,
	tools::slot_key::SlotKey,
	traits::{
		animation::RegisterAnimations,
		handles_bars::HandlesBars,
		handles_effect::HandlesEffect,
		handles_lights::HandlesLights,
		handles_player::{
			ConfiguresPlayerMovement,
			ConfiguresPlayerSkillAnimations,
			HandlesPlayer,
		},
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
	},
};
use components::{
	player::Player,
	player_movement::PlayerMovement,
	skill_animation::SkillAnimation,
};
use std::marker::PhantomData;
use systems::toggle_walk_run::player_toggle_walk_run;

pub struct PlayerPlugin<TAnimation, TPrefabs, TInteractions, TLights, TBars>(
	PhantomData<(TAnimation, TPrefabs, TInteractions, TLights, TBars)>,
);

impl<TAnimation, TPrefabs, TInteractions, TLights, TBars>
	PlayerPlugin<TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	pub fn depends_on(
		_: &TAnimation,
		_: &TPrefabs,
		_: &TInteractions,
		_: &TLights,
		_: &TBars,
	) -> Self {
		Self(PhantomData::<(TAnimation, TPrefabs, TInteractions, TLights, TBars)>)
	}
}

impl<TAnimation, TPrefabs, TInteractions, TLights, TBars> Plugin
	for PlayerPlugin<TAnimation, TPrefabs, TInteractions, TLights, TBars>
where
	TAnimation: Plugin + RegisterAnimations,
	TPrefabs: Plugin + RegisterPrefab,
	TInteractions: Plugin + HandlesEffect<DealDamage, TTarget = Health>,
	TLights: Plugin + HandlesLights,
	TBars: Plugin + HandlesBars,
{
	fn build(&self, app: &mut App) {
		TAnimation::register_animations::<Player>(app);
		TPrefabs::with_dependency::<(TInteractions, TLights, TBars)>()
			.register_prefab::<Player>(app);

		app.add_systems(Update, player_toggle_walk_run).add_systems(
			Update,
			SkillAnimation::system::<TAnimation::TAnimationDispatch>,
		);
	}
}

impl<TAnimation, TPrefabs, TInteractions, TLights, TBars> HandlesPlayer
	for PlayerPlugin<TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TPlayer = Player;
}

impl<TAnimation, TPrefabs, TInteractions, TLights, TBars> ConfiguresPlayerMovement
	for PlayerPlugin<TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TPlayerMovement = PlayerMovement;
}

impl<TAnimation, TPrefabs, TInteractions, TLights, TBars> ConfiguresPlayerSkillAnimations
	for PlayerPlugin<TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TAnimationMarker = SkillAnimation;

	fn start_skill_animation(slot_key: SlotKey) -> Self::TAnimationMarker {
		SkillAnimation::Start(slot_key)
	}

	fn stop_skill_animation() -> Self::TAnimationMarker {
		SkillAnimation::Stop
	}
}
