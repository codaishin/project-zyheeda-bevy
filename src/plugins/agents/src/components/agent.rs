use crate::{
	assets::agent_config::AgentConfigAsset,
	components::{agent_config::AgentConfig, enemy::void_sphere::VoidSphere, player::Player},
};
use bevy::{ecs::system::SystemParamItem, prelude::*};
use common::{
	errors::Unreachable,
	traits::{
		accessors::get::GetProperty,
		handles_enemies::EnemyType,
		handles_map_generation::AgentType,
		load_asset::LoadAsset,
		prefab::{Prefab, PrefabEntityCommands},
		register_derived_component::{DerivableFrom, InsertDerivedComponent},
	},
};
use macros::{SavableComponent, agent_asset};
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[component(immutable)]
#[require(AgentConfig)]
pub(crate) struct Agent {
	pub(crate) agent_type: AgentType,
}

impl GetProperty<AgentType> for Agent {
	fn get_property(&self) -> AgentType {
		self.agent_type
	}
}

impl<TSource> DerivableFrom<'_, '_, TSource> for Agent
where
	TSource: GetProperty<AgentType>,
{
	const INSERT: InsertDerivedComponent = InsertDerivedComponent::IfNew;
	type TParam = ();

	fn derive_from(_: Entity, source: &TSource, _: &SystemParamItem<Self::TParam>) -> Self {
		Self {
			agent_type: source.get_property(),
		}
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
				AgentConfig::<AgentConfigAsset> {
					config_handle: assets.load_asset(agent_asset!("player")),
				},
			)),
			AgentType::Enemy(EnemyType::VoidSphere) => entity.try_insert((
				VoidSphere,
				AgentConfig::<AgentConfigAsset> {
					config_handle: assets.load_asset(agent_asset!("void_sphere")),
				},
			)),
		};

		Ok(())
	}
}
