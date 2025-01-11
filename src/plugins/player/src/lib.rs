pub mod components;

mod resources;
mod systems;

use bevy::prelude::*;
use bevy_rapier3d::plugin::RapierContext;
use common::{
	attributes::health::Health,
	effects::deal_damage::DealDamage,
	states::mouse_context::MouseContext,
	tools::slot_key::SlotKey,
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
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
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

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights>
	PlayerPlugin<(TGameStates, TAnimation, TPrefabs, TInteractions, TLights)>
where
	TGameStates: ThreadSafe + HandlesGameStates,
	TAnimation: ThreadSafe + RegisterAnimations,
	TPrefabs: ThreadSafe + RegisterPrefab,
	TInteractions: ThreadSafe + HandlesEffect<DealDamage, TTarget = Health>,
	TLights: ThreadSafe + HandlesLights,
{
	pub fn depends_on(
		_: &TGameStates,
		_: &TAnimation,
		_: &TPrefabs,
		_: &TInteractions,
		_: &TLights,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights> Plugin
	for PlayerPlugin<(TGameStates, TAnimation, TPrefabs, TInteractions, TLights)>
where
	TGameStates: ThreadSafe + HandlesGameStates,
	TAnimation: ThreadSafe + RegisterAnimations,
	TPrefabs: ThreadSafe + RegisterPrefab,
	TInteractions: ThreadSafe + HandlesEffect<DealDamage, TTarget = Health>,
	TLights: ThreadSafe + HandlesLights,
{
	fn build(&self, app: &mut App) {
		TGameStates::on_starting_new_game(app, Player::spawn);
		TAnimation::register_animations::<Player>(app);
		TPrefabs::with_dependency::<(TInteractions, TLights)>().register_prefab::<Player>(app);

		app.init_state::<MouseContext>()
			.init_resource::<CamRay>()
			.add_systems(
				First,
				(
					set_cam_ray::<Camera, PlayerCamera>,
					set_mouse_hover::<RapierContext>,
				)
					.chain(),
			)
			.add_systems(
				Update,
				SkillAnimation::system::<TAnimation::TAnimationDispatch>,
			)
			.add_systems(Update, player_toggle_walk_run);
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
