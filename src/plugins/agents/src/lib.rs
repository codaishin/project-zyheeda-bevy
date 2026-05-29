mod assets;
mod components;
mod systems;

use crate::{
	assets::agent_meta::{AgentMeta, dto::AgentConfigDto},
	components::{
		agent::{Agent, ApplyAgentAnimations, ApplyAgentConfig},
		agent_config::AgentConfig,
		animate_idle::AnimateIdle,
		enemy::{Enemy, attack_phase::EnemyAttackPhase, void_sphere::VoidSphere},
		player::Player,
		player_camera::PlayerCamera,
	},
};
use bevy::prelude::*;
use common::{
	states::game_state::{GameState, LoadingEssentialAssets},
	systems::{log::OnError, register_animations::RegisterAnimationsSystem},
	traits::{
		after_plugin::AfterPlugin,
		delta::Delta,
		handles_agents::HandlesAgents,
		handles_animations::HandlesAnimations,
		handles_custom_assets::HandlesCustomFolderAssets,
		handles_enemies::HandlesEnemies,
		handles_input::HandlesInput,
		handles_interactive::HandlesInteractive,
		handles_loadout::HandlesLoadout,
		handles_map_generation::HandlesMapGeneration,
		handles_movement::HandlesMovement,
		handles_orientation::HandlesOrientation,
		handles_physics::{HandlesInteractiveDetection, HandlesPhysicsConfig, HandlesRaycast},
		handles_player::{HandlesPlayer, PlayerMainCamera},
		handles_saving::HandlesSaving,
		handles_skill_physics::HandlesPhysicalSkillAgent,
		prefab::AddPrefabObserver,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use std::marker::PhantomData;
use systems::void_sphere::ring_rotation::ring_rotation;

pub struct AgentsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TInput, TSaveGame, TPhysics, TInteractive, TAnimations, TMaps, TMovement, TLoadout>
	AgentsPlugin<(
		TLoading,
		TInput,
		TSaveGame,
		TPhysics,
		TInteractive,
		TAnimations,
		TMaps,
		TMovement,
		TLoadout,
	)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe
		+ HandlesPhysicsConfig
		+ HandlesRaycast
		+ HandlesPhysicalSkillAgent
		+ HandlesInteractiveDetection,
	TInteractive: ThreadSafe + SystemSetDefinition + HandlesInteractive,
	TAnimations: ThreadSafe + HandlesAnimations,
	TMaps: ThreadSafe + HandlesMapGeneration,
	TMovement: ThreadSafe + HandlesMovement + HandlesOrientation,
	TLoadout: ThreadSafe + HandlesLoadout,
{
	#[allow(clippy::too_many_arguments)]
	pub fn from_plugins(
		_: &TLoading,
		_: &TInput,
		_: &TSaveGame,
		_: &TPhysics,
		_: &TInteractive,
		_: &TAnimations,
		_: &TMaps,
		_: &TMovement,
		_: &TLoadout,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TInput, TSaveGame, TPhysics, TInteractive, TAnimations, TMaps, TMovement, TLoadout>
	Plugin
	for AgentsPlugin<(
		TLoading,
		TInput,
		TSaveGame,
		TPhysics,
		TInteractive,
		TAnimations,
		TMaps,
		TMovement,
		TLoadout,
	)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe
		+ HandlesPhysicsConfig
		+ HandlesRaycast
		+ HandlesPhysicalSkillAgent
		+ HandlesInteractiveDetection,
	TInteractive: ThreadSafe + SystemSetDefinition + HandlesInteractive,
	TAnimations: ThreadSafe + HandlesAnimations,
	TMaps: ThreadSafe + HandlesMapGeneration,
	TMovement: ThreadSafe + HandlesMovement + HandlesOrientation,
	TLoadout: ThreadSafe + HandlesLoadout,
{
	fn build(&self, app: &mut App) {
		// # Load Agent
		TLoading::register_custom_folder_assets::<AgentMeta, AgentConfigDto, LoadingEssentialAssets>(
			app,
		);
		app.init_asset::<AgentMeta>();

		app.add_systems(
			Startup,
			Agent::configure_map_prefab::<TMaps::TMapPrefabs>.pipe(OnError::log),
		);
		app.add_systems(
			Update,
			(
				ApplyAgentConfig::system::<
					TLoadout::TLoadoutPrep,
					TPhysics::TAgentMut,
					TMovement::TMovementConfig,
					TPhysics::TConfigMut,
				>,
				ApplyAgentAnimations::register_animations_system::<TAnimations::TAnimationsMut>
					.pipe(OnError::log),
			),
		);

		// # Savedata
		TSaveGame::register_savable_component::<Agent>(app);
		TSaveGame::register_savable_component::<Enemy>(app);
		TSaveGame::register_savable_component::<PlayerCamera>(app);
		TSaveGame::register_savable_component::<EnemyAttackPhase>(app);
		app.register_required_components::<Agent, TSaveGame::TSaveEntityMarker>();

		// # Prefabs
		app.add_prefab_observer::<Agent, ()>();
		app.add_prefab_observer::<VoidSphere, ()>();

		// # Behaviors
		app.register_required_components::<PlayerCamera, TPhysics::TWorldCamera>();
		app.add_systems(
			Update,
			(
				(
					Player::movement::<TInput::TInput, TPhysics::TRaycast, TMovement::TMovementMut>,
					Player::toggle_speed::<TInput::TInput, TMovement::TMovementMut>,
					Player::animate_movement::<TMovement::TMovement, TAnimations::TAnimationsMut>,
					Player::toggle_interactive::<
						TInput::TInput,
						TPhysics::TInteractions,
						TInteractive::TInteractiveMut,
					>,
					Player::use_skills::<
						TInput::TInput,
						TPhysics::TAgentMut,
						TLoadout::TLoadoutActivityMut,
					>,
				)
					.chain(),
				(
					Enemy::attack_decision::<TPhysics::TRaycast>,
					Enemy::chase_decision,
					Enemy::chase_player::<TMovement::TMovementMut>,
					Enemy::animate_movement::<TMovement::TMovement, TAnimations::TAnimationsMut>,
					ring_rotation,
					Enemy::open_doors::<TPhysics::TInteractions, TInteractive::TInteractiveMut>,
					Enemy::begin_attack,
					Enemy::hold_attack::<TPhysics::TAgentMut, TLoadout::TLoadoutActivityMut>,
					Update::delta.pipe(Enemy::advance_attack_phase),
				)
					.chain(),
				AnimateIdle::execute::<TAnimations::TAnimationsMut>,
				AgentConfig::animate_skills::<
					TLoadout::TLoadoutActivity,
					TAnimations::TAnimationsMut,
				>,
			)
				.chain()
				.run_if(in_state(GameState::Play))
				.after_plugin(TInput::SYSTEMS)
				.after_plugin(TMovement::SYSTEMS)
				.after_plugin(TInteractive::SYSTEMS),
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
	type TAgent = AgentConfig;
}
