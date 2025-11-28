mod assets;
mod components;
mod observers;
mod systems;

use crate::{
	assets::agent_config::{AgentConfigAsset, AgentConfigData, dto::AgentConfigAssetDto},
	components::{
		agent::{Agent, tag::AgentTag},
		animate_idle::AnimateIdle,
		enemy::{Enemy, attack_phase::EnemyAttackPhase, void_sphere::VoidSphere},
		movement_config::MovementConfig,
		player::Player,
		player_camera::PlayerCamera,
	},
	observers::agent::{insert_concrete_agent::InsertConcreteAgent, insert_from::InsertFrom},
	systems::agent::insert_model::InsertModelSystem,
};
use bevy::prelude::*;
use common::{
	states::game_state::{GameState, LoadingEssentialAssets},
	systems::log::OnError,
	traits::{
		delta::Delta,
		handles_agents::HandlesAgents,
		handles_animations::{AnimationsSystemParamMut, HandlesAnimations},
		handles_custom_assets::HandlesCustomFolderAssets,
		handles_enemies::HandlesEnemies,
		handles_input::{HandlesInput, InputSystemParam},
		handles_lights::HandlesLights,
		handles_map_generation::HandlesMapGeneration,
		handles_movement::{HandlesMovement, MovementSystemParam, MovementSystemParamMut},
		handles_orientation::{FacingSystemParamMut, HandlesOrientation},
		handles_physics::{HandlesPhysicalAttributes, HandlesRaycast, RaycastSystemParam},
		handles_player::{HandlesPlayer, PlayerMainCamera},
		handles_saving::HandlesSaving,
		handles_skills_control::{HandlesSkillControl, SkillControlParamMut},
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
	TAnimations: ThreadSafe + HandlesAnimations,
	TLights: ThreadSafe + HandlesLights,
	TMaps: ThreadSafe + HandlesMapGeneration,
	TBehaviors: ThreadSafe + HandlesMovement + HandlesOrientation + HandlesSkillControl,
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
	TAnimations: ThreadSafe + HandlesAnimations,
	TLights: ThreadSafe + HandlesLights,
	TMaps: ThreadSafe + HandlesMapGeneration,
	TBehaviors: ThreadSafe + HandlesMovement + HandlesOrientation + HandlesSkillControl,
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
				Agent::register_animations::<AnimationsSystemParamMut<TAnimations>>,
				Agent::<AgentConfigAsset>::insert_attributes::<TPhysics::TDefaultAttributes>,
			),
		);

		// # Savedata
		TSaveGame::register_savable_component::<AgentTag>(app);
		TSaveGame::register_savable_component::<Enemy>(app);
		TSaveGame::register_savable_component::<PlayerCamera>(app);
		TSaveGame::register_savable_component::<MovementConfig>(app);
		TSaveGame::register_savable_component::<EnemyAttackPhase>(app);
		app.register_required_components::<Agent, TSaveGame::TSaveEntityMarker>();

		// # Prefabs
		app.add_prefab_observer::<Player, TLights>();
		app.add_prefab_observer::<VoidSphere, ()>();

		// # Behaviors
		app.register_required_components::<PlayerCamera, TPhysics::TWorldCamera>();
		app.add_observer(Agent::register_skill_spawn_points::<SkillControlParamMut<TBehaviors>>);
		app.add_observer(Player::register_target_definition::<FacingSystemParamMut<TBehaviors>>);
		app.add_observer(Enemy::register_target_definition::<FacingSystemParamMut<TBehaviors>>);
		app.add_systems(
			Update,
			(
				(
					Player::movement::<
						InputSystemParam<TInput>,
						RaycastSystemParam<TPhysics>,
						MovementSystemParamMut<TBehaviors>,
					>,
					Player::toggle_speed::<
						InputSystemParam<TInput>,
						MovementSystemParamMut<TBehaviors>,
					>,
					Player::animate_movement::<
						MovementSystemParam<TBehaviors>,
						AnimationsSystemParamMut<TAnimations>,
					>
						.pipe(OnError::log),
					Player::use_skills::<InputSystemParam<TInput>, SkillControlParamMut<TBehaviors>>,
				)
					.chain(),
				(
					Enemy::attack_decision::<RaycastSystemParam<TPhysics>>,
					Enemy::chase_decision,
					Enemy::chase_player::<MovementSystemParamMut<TBehaviors>>,
					Enemy::animate_movement::<
						MovementSystemParam<TBehaviors>,
						AnimationsSystemParamMut<TAnimations>,
					>
						.pipe(OnError::log),
					ring_rotation,
					Enemy::begin_attack,
					Enemy::hold_attack::<SkillControlParamMut<TBehaviors>>,
					Update::delta.pipe(Enemy::advance_attack_phase),
				)
					.chain(),
				AnimateIdle::execute::<AnimationsSystemParamMut<TAnimations>>,
			)
				.chain()
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

impl<TDependencies> PlayerMainCamera for AgentsPlugin<TDependencies> {
	type TPlayerMainCamera = PlayerCamera;
}

impl<TDependencies> HandlesAgents for AgentsPlugin<TDependencies> {
	type TAgentConfig<'a> = AgentConfigData<'a>;
	type TAgent = Agent;
}
