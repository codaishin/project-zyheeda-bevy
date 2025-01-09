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
		handles_bars::HandlesBars,
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

pub struct PlayerPlugin<TCameras, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>(
	PhantomData<(
		TCameras,
		TGameStates,
		TAnimation,
		TPrefabs,
		TInteractions,
		TLights,
		TBars,
	)>,
);

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
	PlayerPlugin<(), TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
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

impl<TCameras, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> Plugin
	for PlayerPlugin<TCameras, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
where
	TCameras: ThreadSafe + MainCamera + AddPlayerCameras,
	TGameStates: ThreadSafe + HandlesGameStates,
	TAnimation: ThreadSafe + RegisterAnimations,
	TPrefabs: ThreadSafe + RegisterPrefab,
	TInteractions: ThreadSafe + HandlesEffect<DealDamage, TTarget = Health>,
	TLights: ThreadSafe + HandlesLights,
	TBars: ThreadSafe + HandlesBars,
{
	fn build(&self, app: &mut App) {
		TGameStates::on_starting_new_game(app, Player::spawn);
		TAnimation::register_animations::<Player>(app);
		TPrefabs::with_dependency::<(TInteractions, TLights, TBars)>()
			.register_prefab::<Player>(app);
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

impl<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> HandlesPlayer
	for PlayerPlugin<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TPlayer = Player;
}

impl<TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> WithMainCamera
	for PlayerPlugin<(), TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TWithMainCam<TMainCamera>
		= PlayerPlugin<(TMainCamera,), TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
	where
		TMainCamera: Component;

	fn with_main_camera<TMainCamera>(self) -> Self::TWithMainCam<TMainCamera>
	where
		TMainCamera: Component,
	{
		PlayerPlugin(PhantomData)
	}
}

impl<TCameras, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> WithCamera
	for PlayerPlugin<TCameras, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
where
	TCameras: MainCamera + AddPlayerCameras,
{
	type TWithCam<TNewCamera>
		= PlayerPlugin<
		(TCameras, TNewCamera),
		TGameStates,
		TAnimation,
		TPrefabs,
		TInteractions,
		TLights,
		TBars,
	>
	where
		TNewCamera: Component;

	fn with_camera<TNewCamera>(self) -> Self::TWithCam<TNewCamera>
	where
		TNewCamera: Component,
	{
		PlayerPlugin(PhantomData)
	}
}

impl<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> HandlesPlayerCameras
	for PlayerPlugin<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TCamRay = CamRay;
}

impl<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> HandlesPlayerMouse
	for PlayerPlugin<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TMouseHover = MouseHover;
}

impl<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars> ConfiguresPlayerMovement
	for PlayerPlugin<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TPlayerMovement = PlayerMovement;
}

impl<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
	ConfiguresPlayerSkillAnimations
	for PlayerPlugin<T, TGameStates, TAnimation, TPrefabs, TInteractions, TLights, TBars>
{
	type TAnimationMarker = SkillAnimation;

	fn start_skill_animation(slot_key: SlotKey) -> Self::TAnimationMarker {
		SkillAnimation::Start(slot_key)
	}

	fn stop_skill_animation() -> Self::TAnimationMarker {
		SkillAnimation::Stop
	}
}
