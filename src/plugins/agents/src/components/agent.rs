use crate::{
	assets::agent_meta::AgentMeta,
	components::{agent_config::AgentConfig, enemy::void_sphere::VoidSphere, player::Player},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{prelude::*, systems::register_animations::AnimationsMarker};
use macros::{SavableComponent, asset_path, serde_model};

#[serde_model]
#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy)]
#[component(immutable)]
#[savable_component(id = "agent")]
#[require(AgentConfig, ApplyAgentModel, Transform)]
pub struct Agent {
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

	pub(crate) fn configure_map_prefab<TMapGeneration>(
		mut new_agent: StaticSystemParam<TMapGeneration>,
	) where
		TMapGeneration: for<'c> GetContextMut<AgentPrefabs, TContext<'c>: SetPrefab<AgentType>>,
	{
		TMapGeneration::get_context_mut(&mut new_agent, MapPrefabs::KEY)
			.set_prefab(Self::map_prefab);
	}
}

impl View<AgentType> for Agent {
	fn view(&self) -> AgentType {
		self.agent_type
	}
}

impl Prefab<()> for Agent {
	type TError = Unreachable;
	type TSystemParam = Res<'static, AssetServer>;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		assets: StaticSystemParam<Res<AssetServer>>,
	) -> Result<(), Self::TError> {
		let path = match self.agent_type {
			AgentType::Player => {
				entity.try_insert(Player);
				asset_path!("agents/player/meta.agent")
			}
			AgentType::Enemy(EnemyType::VoidSphere) => {
				entity.try_insert(VoidSphere);
				asset_path!("agents/void_sphere/meta.agent")
			}
		};

		entity.try_insert(AgentConfig {
			config_handle: assets.load(path),
		});

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct ApplyAgentModel;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct ApplyAgentAnimations;

impl AnimationsMarker for ApplyAgentAnimations {
	type TConfig = AgentMeta;
	type TConfigComponent = AgentConfig;
}

#[derive(Component, Debug, PartialEq)]
pub(crate) struct AgentTransformDirty;
