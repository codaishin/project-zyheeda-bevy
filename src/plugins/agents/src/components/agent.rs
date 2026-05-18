use crate::components::{
	agent_config::AgentConfig,
	enemy::void_sphere::VoidSphere,
	player::Player,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	errors::{ErrorData, Level, Unreachable},
	traits::{
		accessors::get::{GetContextMut, View},
		handles_enemies::EnemyType,
		handles_map_generation::{AgentType, GroundPosition, MapPrefabs, SetPrefab},
		prefab::{Prefab, PrefabEntityCommands},
	},
	zyheeda_commands::ZyheedaEntityCommands,
};
use macros::{SavableComponent, asset_path};
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
	fn map_prefab(
		mut entity: ZyheedaEntityCommands,
		ground_position: GroundPosition,
		agent_type: AgentType,
	) {
		entity.try_insert((
			Transform::from_translation(*ground_position),
			Agent { agent_type },
			AgentTransformDirty,
		));
	}

	pub(crate) fn configure_map_prefab<TNewMapAgent>(
		mut new_agent: StaticSystemParam<TNewMapAgent>,
	) -> Result<(), NoPrefabContext>
	where
		TNewMapAgent:
			for<'c> GetContextMut<MapPrefabs<AgentType>, TContext<'c>: SetPrefab<AgentType>>,
	{
		let Some(mut ctx) = TNewMapAgent::get_context_mut(&mut new_agent, MapPrefabs::KEY) else {
			return Err(NoPrefabContext);
		};

		ctx.set_map_agent_prefab(Self::map_prefab);

		Ok(())
	}
}

impl View<AgentType> for Agent {
	fn view(&self) -> AgentType {
		self.agent_type
	}
}

impl Prefab<()> for Agent {
	type TError = Unreachable;
	type TSystemParam<'w, 's> = Res<'w, AssetServer>;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		assets: StaticSystemParam<Res<AssetServer>>,
	) -> Result<(), Self::TError> {
		match self.agent_type {
			AgentType::Player => entity.try_insert((
				Player,
				AgentConfig {
					config_handle: assets.load(asset_path!("agents/player/meta.agent")),
				},
			)),
			AgentType::Enemy(EnemyType::VoidSphere) => entity.try_insert((
				VoidSphere,
				AgentConfig {
					config_handle: assets.load(asset_path!("agents/void_sphere/meta.agent")),
				},
			)),
		};

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct ApplyAgentConfig;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct ApplyAgentAnimations;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct AgentTransformDirty;

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
