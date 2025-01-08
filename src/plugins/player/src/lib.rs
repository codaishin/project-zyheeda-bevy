pub mod components;

mod resources;
mod systems;
mod traits;

use bevy::prelude::*;
use bevy_rapier3d::plugin::RapierContext;
use common::{
	attributes::health::Health,
	components::MainCamera,
	effects::deal_damage::DealDamage,
	states::mouse_context::MouseContext,
	tools::slot_key::SlotKey,
	traits::{
		animation::RegisterAnimations,
		handles_bars::HandlesBars,
		handles_effect::HandlesEffect,
		handles_game_states::HandlesGameStates,
		handles_lights::HandlesLights,
		handles_player::{
			ConfiguresPlayerMovement,
			ConfiguresPlayerSkillAnimations,
			HandlesPlayer,
			HandlesPlayerCam,
			HandlesPlayerMouse,
		},
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
	},
};
use components::{
	player::Player,
	player_movement::PlayerMovement,
	skill_animation::SkillAnimation,
};
use resources::{cam_ray::CamRay, mouse_hover::MouseHover};
use std::marker::PhantomData;
use systems::{
	set_cam_ray::set_cam_ray,
	set_mouse_hover::set_mouse_hover,
	toggle_walk_run::player_toggle_walk_run,
};

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

		app.init_state::<MouseContext>()
			.init_resource::<CamRay>()
			.add_systems(
				Update,
				SkillAnimation::system::<TAnimation::TAnimationDispatch>,
			)
			.add_systems(
				First,
				(
					set_cam_ray::<Camera, MainCamera>,
					set_mouse_hover::<RapierContext>,
				)
					.chain(),
			)
			.add_systems(Update, player_toggle_walk_run);
	}
}

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> HandlesPlayer
	for PlayerPlugin<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TPlayer = Player;
}

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> HandlesPlayerCam
	for PlayerPlugin<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TCamRay = CamRay;
}

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> HandlesPlayerMouse
	for PlayerPlugin<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TMouseHover = MouseHover;
}

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> ConfiguresPlayerMovement
	for PlayerPlugin<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TPlayerMovement = PlayerMovement;
}

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
	ConfiguresPlayerSkillAnimations
	for PlayerPlugin<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TAnimationMarker = SkillAnimation;

	fn start_skill_animation(slot_key: SlotKey) -> Self::TAnimationMarker {
		SkillAnimation::Start(slot_key)
	}

	fn stop_skill_animation() -> Self::TAnimationMarker {
		SkillAnimation::Stop
	}
}
