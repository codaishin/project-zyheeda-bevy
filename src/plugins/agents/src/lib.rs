mod assets;
mod components;
mod systems;

use crate::{
	assets::agent_config::{AgentConfigAsset, dto::AgentConfigDto},
	components::{
		agent::Agent,
		agent_config::{
			AgentConfig,
			InsertAgentDefaultLoadout,
			InsertAgentModel,
			RegisterAgentAnimations,
			RegisterAgentLoadoutBones,
			RegisterSkillSpawnPoints,
		},
		animate_idle::AnimateIdle,
		enemy::{Enemy, attack_phase::EnemyAttackPhase, void_sphere::VoidSphere},
		movement_config::MovementConfig,
		player::Player,
		player_camera::PlayerCamera,
	},
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
		handles_loadout::{
			HandlesLoadout,
			LoadoutActivityMutParam,
			LoadoutActivityParam,
			LoadoutPrepParam,
		},
		handles_map_generation::HandlesMapGeneration,
		handles_movement::{HandlesMovement, MovementSystemParam, MovementSystemParamMut},
		handles_orientation::{FacingSystemParamMut, HandlesOrientation},
		handles_physics::{
			HandlesPhysicalAttributes,
			HandlesPhysicalEffectTargets,
			HandlesRaycast,
			RaycastSystemParam,
			physical_bodies::HandlesPhysicalBodies,
		},
		handles_player::{HandlesPlayer, PlayerMainCamera},
		handles_saving::HandlesSaving,
		handles_skill_physics::{HandlesPhysicalSkillSpawnPoints, SkillSpawnPointsMut},
		prefab::AddPrefabObserver,
		register_derived_component::RegisterDerivedComponent,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use std::marker::PhantomData;
use systems::void_sphere::ring_rotation::ring_rotation;

pub struct AgentsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading, TInput, TSaveGame, TPhysics, TAnimations, TMaps, TBehaviors, TLoadout>
	AgentsPlugin<(
		TLoading,
		TInput,
		TSaveGame,
		TPhysics,
		TAnimations,
		TMaps,
		TBehaviors,
		TLoadout,
	)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe
		+ HandlesPhysicalEffectTargets
		+ HandlesPhysicalAttributes
		+ HandlesPhysicalBodies
		+ HandlesRaycast
		+ HandlesPhysicalSkillSpawnPoints,
	TAnimations: ThreadSafe + HandlesAnimations,
	TMaps: ThreadSafe + HandlesMapGeneration,
	TBehaviors: ThreadSafe + HandlesMovement + HandlesOrientation,
	TLoadout: ThreadSafe + HandlesLoadout,
{
	#[allow(clippy::too_many_arguments)]
	pub fn from_plugins(
		_: &TLoading,
		_: &TInput,
		_: &TSaveGame,
		_: &TPhysics,
		_: &TAnimations,
		_: &TMaps,
		_: &TBehaviors,
		_: &TLoadout,
	) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading, TInput, TSaveGame, TPhysics, TAnimations, TMaps, TBehaviors, TLoadout> Plugin
	for AgentsPlugin<(
		TLoading,
		TInput,
		TSaveGame,
		TPhysics,
		TAnimations,
		TMaps,
		TBehaviors,
		TLoadout,
	)>
where
	TLoading: ThreadSafe + HandlesCustomFolderAssets,
	TInput: ThreadSafe + SystemSetDefinition + HandlesInput,
	TSaveGame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe
		+ HandlesPhysicalEffectTargets
		+ HandlesPhysicalAttributes
		+ HandlesPhysicalBodies
		+ HandlesRaycast
		+ HandlesPhysicalSkillSpawnPoints,
	TAnimations: ThreadSafe + HandlesAnimations,
	TMaps: ThreadSafe + HandlesMapGeneration,
	TBehaviors: ThreadSafe + HandlesMovement + HandlesOrientation,
	TLoadout: ThreadSafe + HandlesLoadout,
{
	fn build(&self, app: &mut App) {
		// # Load Agent
		TLoading::register_custom_folder_assets::<
			AgentConfigAsset,
			AgentConfigDto,
			LoadingEssentialAssets,
		>(app);
		app.init_asset::<AgentConfigAsset>();

		TPhysics::mark_as_effect_target::<Agent>(app);
		app.register_derived_component::<TMaps::TNewWorldAgent, Agent>();
		app.add_systems(
			Update,
			(
				AgentConfig::<AgentConfigAsset>::insert_attributes::<TPhysics::TDefaultAttributes>,
				InsertAgentModel::execute,
				InsertAgentDefaultLoadout::execute::<AgentConfigAsset, LoadoutPrepParam<TLoadout>>,
				RegisterAgentLoadoutBones::execute::<LoadoutPrepParam<TLoadout>>,
				RegisterSkillSpawnPoints::execute::<SkillSpawnPointsMut<TPhysics>>,
				RegisterAgentAnimations::execute::<AnimationsSystemParamMut<TAnimations>>,
			),
		);

		// # Savedata
		TSaveGame::register_savable_component::<Agent>(app);
		TSaveGame::register_savable_component::<Enemy>(app);
		TSaveGame::register_savable_component::<PlayerCamera>(app);
		TSaveGame::register_savable_component::<MovementConfig>(app);
		TSaveGame::register_savable_component::<EnemyAttackPhase>(app);
		app.register_required_components::<Agent, TSaveGame::TSaveEntityMarker>();

		// # Prefabs
		app.add_prefab_observer::<Agent, ()>();
		app.add_prefab_observer::<Player, TPhysics>();
		app.add_prefab_observer::<VoidSphere, TPhysics>();

		// # Behaviors
		app.register_required_components::<PlayerCamera, TPhysics::TWorldCamera>();
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
					Player::use_skills::<InputSystemParam<TInput>, LoadoutActivityMutParam<TLoadout>>,
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
					Enemy::hold_attack::<LoadoutActivityMutParam<TLoadout>>,
					Update::delta.pipe(Enemy::advance_attack_phase),
				)
					.chain(),
				AnimateIdle::execute::<AnimationsSystemParamMut<TAnimations>>,
				AgentConfig::animate_skills::<
					LoadoutActivityParam<TLoadout>,
					AnimationsSystemParamMut<TAnimations>,
				>
					.pipe(OnError::log),
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
	type TAgent = AgentConfig;
}
