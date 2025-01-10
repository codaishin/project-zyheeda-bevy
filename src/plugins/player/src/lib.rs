pub mod components;

mod resources;
mod systems;
mod traits;

use bevy::prelude::*;
use bevy_rapier3d::plugin::RapierContext;
use common::{
	attributes::health::Health,
	effects::deal_damage::DealDamage,
	states::{game_state::GameState, mouse_context::MouseContext},
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
			WithCamera,
			WithMainCamera,
		},
		prefab::{RegisterPrefab, RegisterPrefabWithDependency},
		thread_safe::ThreadSafe,
	},
};
use components::{
	orbit_player::OrbitPlayer,
	player::Player,
	player_movement::PlayerMovement,
	skill_animation::SkillAnimation,
};
use resources::{cam_ray::CamRay, mouse_hover::MouseHover};
use std::marker::PhantomData;
use systems::{
	move_on_orbit::move_on_orbit,
	move_with_target::move_with_target,
	set_cam_ray::set_cam_ray,
	set_mouse_hover::set_mouse_hover,
	toggle_walk_run::player_toggle_walk_run,
};
use traits::{add_player_camera::AddPlayerCameras, main_camera::MainCamera};

pub struct PlayerPlugin<TCameras, TGameStates, TAnimation, TPrefabs, TInteractions, TLights>(
	PhantomData<(
		TCameras,
		TGameStates,
		TAnimation,
		TPrefabs,
		TInteractions,
		TLights,
	)>,
);

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights>
	PlayerPlugin<(), TGameStates, TAnimation, TPrefabs, TInteractions, TLights>
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

impl<TCameras, TGameStates, TAnimation, TPrefabs, TInteractions, TLights> Plugin
	for PlayerPlugin<TCameras, TGameStates, TAnimation, TPrefabs, TInteractions, TLights>
where
	TCameras: ThreadSafe + MainCamera + AddPlayerCameras,
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
		TCameras::add_player_cameras(app);

		app.init_state::<MouseContext>()
			.init_resource::<CamRay>()
			.add_systems(
				Update,
				SkillAnimation::system::<TAnimation::TAnimationDispatch>,
			)
			.add_systems(
				First,
				(
					set_cam_ray::<Camera, TCameras::TMainCamera>,
					set_mouse_hover::<RapierContext>,
				)
					.chain(),
			)
			.add_systems(
				Update,
				(
					move_on_orbit::<OrbitPlayer>,
					move_with_target::<OrbitPlayer>,
				)
					.run_if(in_state(GameState::Play)),
			)
			.add_systems(Update, player_toggle_walk_run);
	}
}

impl<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights> HandlesPlayer
	for PlayerPlugin<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights>
{
	type TPlayer = Player;
}

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights> WithMainCamera
	for PlayerPlugin<(), TGameStates, TAnimation, TPrefabs, TInteractions, TLights>
{
	type TWithMainCam<TMainCamera>
		= PlayerPlugin<(TMainCamera,), TGameStates, TAnimation, TPrefabs, TInteractions, TLights>
	where
		TMainCamera: Component;

	fn with_main_camera<TMainCamera>(self) -> Self::TWithMainCam<TMainCamera>
	where
		TMainCamera: Component,
	{
		PlayerPlugin(PhantomData)
	}
}

impl<TCameras, TGameStates, TAnimation, TPrefabs, TInteractions, TLights> WithCamera
	for PlayerPlugin<TCameras, TGameStates, TAnimation, TPrefabs, TInteractions, TLights>
where
	TCameras: MainCamera + AddPlayerCameras,
{
	type TWithCam<TNewCamera>
		= PlayerPlugin<(TCameras, TNewCamera), TGameStates, TAnimation, TPrefabs, TInteractions, TLights>
	where
		TNewCamera: Component;

	fn with_camera<TNewCamera>(self) -> Self::TWithCam<TNewCamera>
	where
		TNewCamera: Component,
	{
		PlayerPlugin(PhantomData)
	}
}

impl<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights> HandlesPlayerCameras
	for PlayerPlugin<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights>
{
	type TCamRay = CamRay;
}

impl<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights> HandlesPlayerMouse
	for PlayerPlugin<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights>
{
	type TMouseHover = MouseHover;
}

impl<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights> ConfiguresPlayerMovement
	for PlayerPlugin<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights>
{
	type TPlayerMovement = PlayerMovement;
}

impl<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights> ConfiguresPlayerSkillAnimations
	for PlayerPlugin<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights>
{
	type TAnimationMarker = SkillAnimation;

	fn start_skill_animation(slot_key: SlotKey) -> Self::TAnimationMarker {
		SkillAnimation::Start(slot_key)
	}

	fn stop_skill_animation() -> Self::TAnimationMarker {
		SkillAnimation::Stop
	}
}
