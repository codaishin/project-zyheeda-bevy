mod assets;
mod components;
mod observers;
mod resources;
mod systems;

use crate::{
	assets::agent_config::{AgentConfigAsset, AgentConfigData, dto::AgentConfigAssetDto},
	components::{
		agent::{Agent, tag::AgentTag},
		enemy::{Enemy, void_sphere::VoidSphere},
		player::Player,
		player_camera::PlayerCamera,
		player_movement::PlayerMovement,
		skill_animation::SkillAnimation,
	},
	resources::{cam_ray::CamRay, mouse_hover::MouseHover},
	systems::{
		agent::insert_model::InsertModelSystem,
		set_cam_ray::set_cam_ray,
		set_mouse_hover::set_mouse_hover,
		toggle_walk_run::player_toggle_walk_run,
	},
};
use bevy::prelude::*;
use common::{
	states::game_state::{GameState, LoadingEssentialAssets},
	tools::action_key::slot::{NoValidAgentKey, PlayerSlot, SlotKey},
	traits::{
		animation::RegisterAnimations,
		handles_agents::HandlesAgents,
		handles_custom_assets::HandlesCustomFolderAssets,
		handles_enemies::HandlesEnemies,
		handles_input::{HandlesInput, InputSystemParam},
		handles_lights::HandlesLights,
		handles_physics::HandlesPhysicalAttributes,
		handles_player::{
			ConfiguresPlayerMovement,
			ConfiguresPlayerSkillAnimations,
			HandlesPlayer,
			HandlesPlayerCameras,
			HandlesPlayerMouse,
			PlayerMainCamera,
		},
		handles_saving::HandlesSaving,
		prefab::AddPrefabObserver,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use std::marker::PhantomData;
use systems::void_sphere::ring_rotation::ring_rotation;

pub struct AgentsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TInput, TSaveGame, TPhysics, TAnimations, TLights>
	AgentsPlugin<(TLoading, TInput, TSaveGame, TPhysics, TAnimations, TLights)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesPhysicalAttributes,
	TAnimations: ThreadSafe + RegisterAnimations,
	TLights: ThreadSafe + HandlesLights,
{
	pub fn from_plugins(
		_: &TLoading,
		_: &TInput,
		_: &TSaveGame,
		_: &TPhysics,
		_: &TAnimations,
		_: &TLights,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TInput, TSaveGame, TPhysics, TAnimations, TLights> Plugin
	for AgentsPlugin<(TLoading, TInput, TSaveGame, TPhysics, TAnimations, TLights)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesPhysicalAttributes,
	TAnimations: ThreadSafe + RegisterAnimations,
	TLights: ThreadSafe + HandlesLights,
{
	fn build(&self, app: &mut App) {
		// Load Agent
		TLoading::register_custom_folder_assets::<
			AgentConfigAsset,
			AgentConfigAssetDto,
			LoadingEssentialAssets,
		>(app);
		app.init_asset::<AgentConfigAsset>();
		app.add_observer(Agent::insert_self_from::<AgentTag>);
		app.add_systems(
			Update,
			(
				Agent::insert_model,
				Agent::<AgentConfigAsset>::insert_attributes::<TPhysics::TDefaultAttributes>,
			),
		);

		// Animations
		TAnimations::register_animations::<Player>(app);
		app.add_systems(
			Update,
			(
				ring_rotation,
				SkillAnimation::system::<TAnimations::TAnimationDispatch>,
			)
				.run_if(in_state(GameState::Play)),
		);

		// Savedata
		TSaveGame::register_savable_component::<AgentTag>(app);
		TSaveGame::register_savable_component::<Enemy>(app);
		TSaveGame::register_savable_component::<PlayerCamera>(app);
		TSaveGame::register_savable_component::<PlayerMovement>(app);
		app.register_required_components::<AgentTag, TSaveGame::TSaveEntityMarker>();

		// Prefabs
		app.add_prefab_observer::<Player, TLights>();
		app.add_prefab_observer::<VoidSphere, ()>();

		// Behaviors
		app.init_resource::<CamRay>();
		app.init_resource::<MouseHover>();
		app.add_systems(
			First,
			(set_cam_ray::<Camera, PlayerCamera>, set_mouse_hover)
				.chain()
				.run_if(in_state(GameState::Play)),
		);
		app.add_systems(
			Update,
			player_toggle_walk_run::<InputSystemParam<TInput>>
				.run_if(in_state(GameState::Play))
				.after(TInput::SYSTEMS),
		);
	}
}

impl<TDependencies> HandlesEnemies for AgentsPlugin<TDependencies> {
	type TEnemy = Enemy;
}

impl<TDependencies> HandlesPlayer for AgentsPlugin<TDependencies> {
	type TPlayer = Player;
}

impl<TDependencies> HandlesPlayerCameras for AgentsPlugin<TDependencies> {
	type TCamRay = CamRay;
}

impl<TDependencies> HandlesPlayerMouse for AgentsPlugin<TDependencies> {
	type TMouseHover = MouseHover;
}

impl<TDependencies> ConfiguresPlayerMovement for AgentsPlugin<TDependencies> {
	type TPlayerMovement = PlayerMovement;
}

impl<TDependencies> ConfiguresPlayerSkillAnimations for AgentsPlugin<TDependencies> {
	type TAnimationMarker = SkillAnimation;
	type TError = NoValidAgentKey<PlayerSlot>;

	fn start_skill_animation(slot_key: SlotKey) -> Result<Self::TAnimationMarker, Self::TError> {
		Ok(SkillAnimation::Start(PlayerSlot::try_from(slot_key)?))
	}

	fn stop_skill_animation() -> Self::TAnimationMarker {
		SkillAnimation::Stop
	}
}

impl<TDependencies> PlayerMainCamera for AgentsPlugin<TDependencies> {
	type TPlayerMainCamera = PlayerCamera;
}

impl<TDependencies> HandlesAgents for AgentsPlugin<TDependencies> {
	type TAgentConfig<'a> = AgentConfigData<'a>;
	type TAgent = Agent;
}
