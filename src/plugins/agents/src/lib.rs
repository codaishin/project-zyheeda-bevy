mod assets;
mod components;
mod observers;
mod systems;

use crate::{
	assets::agent_config::{AgentConfigAsset, AgentConfigData, dto::AgentConfigAssetDto},
	components::{
		agent::{Agent, tag::AgentTag},
		enemy::{Enemy, void_sphere::VoidSphere},
		movement_config::MovementConfig,
		player::Player,
		player_camera::PlayerCamera,
		skill_animation::SkillAnimation,
	},
	observers::agent::{insert_concrete_agent::InsertConcreteAgent, insert_from::InsertFrom},
	systems::agent::insert_model::InsertModelSystem,
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
		handles_map_generation::HandlesMapGeneration,
		handles_movement::{HandlesMovement, MovementSystemParamMut},
		handles_physics::{HandlesPhysicalAttributes, HandlesRaycast, RaycastSystemParam},
		handles_player::{ConfiguresPlayerSkillAnimations, HandlesPlayer, PlayerMainCamera},
		handles_saving::HandlesSaving,
		handles_skills_control::{HandlesSKillControl, SKillControlParamMut},
		prefab::AddPrefabObserver,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use std::marker::PhantomData;
use systems::void_sphere::ring_rotation::ring_rotation;

pub struct AgentsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TInput, TSaveGame, TPhysics, TAnimations, TLights, TMaps, TBehaviors>
	AgentsPlugin<(
		TLoading,
		TInput,
		TSaveGame,
		TPhysics,
		TAnimations,
		TLights,
		TMaps,
		TBehaviors,
	)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesPhysicalAttributes + HandlesRaycast,
	TAnimations: ThreadSafe + RegisterAnimations,
	TLights: ThreadSafe + HandlesLights,
	TMaps: ThreadSafe + HandlesMapGeneration,
	TBehaviors: ThreadSafe + HandlesMovement + HandlesSKillControl,
{
	#[allow(clippy::too_many_arguments)]
	pub fn from_plugins(
		_: &TLoading,
		_: &TInput,
		_: &TSaveGame,
		_: &TPhysics,
		_: &TAnimations,
		_: &TLights,
		_: &TMaps,
		_: &TBehaviors,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TInput, TSaveGame, TPhysics, TAnimations, TLights, TMaps, TBehaviors> Plugin
	for AgentsPlugin<(
		TLoading,
		TInput,
		TSaveGame,
		TPhysics,
		TAnimations,
		TLights,
		TMaps,
		TBehaviors,
	)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + HandlesPhysicalAttributes + HandlesRaycast,
	TAnimations: ThreadSafe + RegisterAnimations,
	TLights: ThreadSafe + HandlesLights,
	TMaps: ThreadSafe + HandlesMapGeneration,
	TBehaviors: ThreadSafe + HandlesMovement + HandlesSKillControl,
{
	fn build(&self, app: &mut App) {
		// # Load Agent
		TLoading::register_custom_folder_assets::<
			AgentConfigAsset,
			AgentConfigAssetDto,
			LoadingEssentialAssets,
		>(app);
		app.init_asset::<AgentConfigAsset>();

		// Using `AgentTag` to buffer the map agent type, in case `TNewWorldAgent` is not
		// persistent across game sessions
		app.add_observer(AgentTag::insert_from::<TMaps::TNewWorldAgent>);
		app.add_observer(Agent::insert_from::<AgentTag>);
		app.add_observer(Agent::insert_concrete_agent);
		app.add_systems(
			Update,
			(
				Agent::insert_model,
				Agent::<AgentConfigAsset>::insert_attributes::<TPhysics::TDefaultAttributes>,
			),
		);

		// # Animations
		TAnimations::register_animations::<Player>(app);
		app.add_systems(
			Update,
			(
				ring_rotation,
				SkillAnimation::system::<TAnimations::TAnimationDispatch>,
			)
				.run_if(in_state(GameState::Play)),
		);

		// # Savedata
		TSaveGame::register_savable_component::<AgentTag>(app);
		TSaveGame::register_savable_component::<Enemy>(app);
		TSaveGame::register_savable_component::<PlayerCamera>(app);
		TSaveGame::register_savable_component::<MovementConfig>(app);
		app.register_required_components::<Agent, TSaveGame::TSaveEntityMarker>();

		// # Prefabs
		app.add_prefab_observer::<Player, TLights>();
		app.add_prefab_observer::<VoidSphere, ()>();

		// # Behaviors
		app.register_required_components::<PlayerCamera, TPhysics::TWorldCamera>();
		app.add_observer(Agent::register_skill_spawn_points::<SKillControlParamMut<TBehaviors>>);
		app.add_systems(
			Update,
			(
				Player::movement::<
					InputSystemParam<TInput>,
					RaycastSystemParam<TPhysics>,
					MovementSystemParamMut<TBehaviors>,
				>,
				Player::toggle_speed::<InputSystemParam<TInput>, MovementSystemParamMut<TBehaviors>>,
				Player::use_skills::<InputSystemParam<TInput>, SKillControlParamMut<TBehaviors>>,
				Enemy::chase_player::<MovementSystemParamMut<TBehaviors>>,
			)
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
