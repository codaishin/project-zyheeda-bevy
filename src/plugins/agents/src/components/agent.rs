use crate::components::{
	agent_config::AgentConfig,
	enemy::void_sphere::VoidSphere,
	player::Player,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	errors::{ErrorData, Level, Unreachable},
	traits::{
		accessors::get::{GetContextMut, GetProperty},
		handles_enemies::EnemyType,
		handles_map_generation::{AgentPrefab, AgentType, GroundPosition, SetMapAgentPrefab},
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
	},
	zyheeda_commands::ZyheedaEntityCommands,
};
use macros::{SavableComponent, agent_asset};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[component(immutable)]
#[savable_component(id = "agent")]
#[require(AgentConfig, ApplyAgentConfig, Transform)]
pub(crate) struct Agent {
	pub(crate) agent_type: AgentType,
}

impl Agent {
	fn prefab_essentials(
		mut entity: ZyheedaEntityCommands,
		translation: GroundPosition,
		agent_type: AgentType,
	) {
		entity.try_insert((
			Transform {
				translation,
				..default()
			},
			Agent { agent_type },
		));
	}

	pub(crate) fn set_prefab_essentials<TNewAgent>(
		mut new_agent: StaticSystemParam<TNewAgent>,
	) -> Result<(), NoPrefabContext>
	where
		TNewAgent: for<'c> GetContextMut<AgentPrefab, TContext<'c>: SetMapAgentPrefab>,
	{
		let Some(mut ctx) = TNewAgent::get_context_mut(&mut new_agent, AgentPrefab) else {
			return Err(NoPrefabContext);
		};

		ctx.set_map_agent_prefab(Self::prefab_essentials);

		Ok(())
	}
}

impl GetProperty<AgentType> for Agent {
	fn get_property(&self) -> AgentType {
		self.agent_type
	}
}

impl Prefab<()> for Agent {
	type TError = Unreachable;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		assets: &mut impl LoadAsset,
	) -> Result<(), Self::TError> {
		match self.agent_type {
			AgentType::Player => entity.try_insert((
				Player,
				AgentConfig {
					config_handle: assets.load_asset(agent_asset!("player")),
				},
			)),
			AgentType::Enemy(EnemyType::VoidSphere) => entity.try_insert((
				VoidSphere,
				AgentConfig {
					config_handle: assets.load_asset(agent_asset!("void_sphere")),
				},
			)),
		};

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct ApplyAgentConfig;

#[derive(Debug, PartialEq)]
pub struct NoPrefabContext;

impl Display for NoPrefabContext {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Cannot set agent prefab due to missing prefab context in map plugin"
		)
	}
}

impl ErrorData for NoPrefabContext {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl std::fmt::Display {
		"No Prefab Context"
	}

	fn into_details(self) -> impl std::fmt::Display {
		self
	}
}
