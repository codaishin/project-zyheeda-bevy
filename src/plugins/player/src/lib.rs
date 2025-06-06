pub mod components;

mod resources;
mod systems;

use bevy::prelude::*;
use common::{
	attributes::{affected_by::AffectedBy, health::Health},
	effects::{deal_damage::DealDamage, force_shield::Force, gravity::Gravity},
	states::game_state::GameState,
	tools::action_key::{movement::MovementKey, slot::SlotKey},
	traits::{
		animation::RegisterAnimations,
		handles_effect::HandlesEffect,
		handles_game_states::HandlesGameStates,
		handles_lights::HandlesLights,
		handles_player::{
			ConfiguresPlayerMovement,
			ConfiguresPlayerSkillAnimations,
			HandlesPlayer,
			HandlesPlayerCameras,
			HandlesPlayerMouse,
			PlayerMainCamera,
		},
		handles_settings::HandlesSettings,
		prefab::AddPrefabObserver,
		thread_safe::ThreadSafe,
	},
};
use components::{
	player::Player,
	player_camera::PlayerCamera,
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

pub struct PlayerPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSettings, TGameStates, TAnimations, TInteractions, TLights>
	PlayerPlugin<(TSettings, TGameStates, TAnimations, TInteractions, TLights)>
where
	TSettings: ThreadSafe + HandlesSettings,
	TGameStates: ThreadSafe + HandlesGameStates,
	TAnimations: ThreadSafe + RegisterAnimations,
	TInteractions: ThreadSafe + HandlesEffect<DealDamage, TTarget = Health>,
	TLights: ThreadSafe + HandlesLights,
{
	pub fn from_plugins(
		_: &TSettings,
		_: &TGameStates,
		_: &TAnimations,
		_: &TInteractions,
		_: &TLights,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TSettings, TGameStates, TAnimations, TInteractions, TLights> Plugin
	for PlayerPlugin<(TSettings, TGameStates, TAnimations, TInteractions, TLights)>
where
	TSettings: ThreadSafe + HandlesSettings,
	TGameStates: ThreadSafe + HandlesGameStates,
	TAnimations: ThreadSafe + RegisterAnimations,
	TInteractions: ThreadSafe
		+ HandlesEffect<DealDamage, TTarget = Health>
		+ HandlesEffect<Gravity, TTarget = AffectedBy<Gravity>>
		+ HandlesEffect<Force, TTarget = AffectedBy<Force>>,
	TLights: ThreadSafe + HandlesLights,
{
	fn build(&self, app: &mut App) {
		TGameStates::on_starting_new_game(app, Player::spawn);
		TAnimations::register_animations::<Player>(app);

		app.init_resource::<CamRay>()
			.add_prefab_observer::<Player, (TInteractions, TLights)>()
			.add_systems(
				First,
				(set_cam_ray::<Camera, PlayerCamera>, set_mouse_hover).chain(),
			)
			.add_systems(
				Update,
				(
					SkillAnimation::system::<TAnimations::TAnimationDispatch>,
					player_toggle_walk_run::<TSettings::TKeyMap<MovementKey>>,
				)
					.run_if(not(in_state(GameState::LoadingEssentialAssets))),
			);
	}
}

impl<TDependencies> HandlesPlayer for PlayerPlugin<TDependencies> {
	type TPlayer = Player;
}

impl<TDependencies> HandlesPlayerCameras for PlayerPlugin<TDependencies> {
	type TCamRay = CamRay;
}

impl<TDependencies> HandlesPlayerMouse for PlayerPlugin<TDependencies> {
	type TMouseHover = MouseHover;
}

impl<TDependencies> ConfiguresPlayerMovement for PlayerPlugin<TDependencies> {
	type TPlayerMovement = PlayerMovement;
}

impl<TDependencies> ConfiguresPlayerSkillAnimations for PlayerPlugin<TDependencies> {
	type TAnimationMarker = SkillAnimation;

	fn start_skill_animation(slot_key: SlotKey) -> Self::TAnimationMarker {
		SkillAnimation::Start(slot_key)
	}

	fn stop_skill_animation() -> Self::TAnimationMarker {
		SkillAnimation::Stop
	}
}

impl<TDependencies> PlayerMainCamera for PlayerPlugin<TDependencies> {
	type TPlayerMainCamera = PlayerCamera;
}
