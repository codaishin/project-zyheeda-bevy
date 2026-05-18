use crate::resources::agents::prefab::PrefabRegister;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::GetContextMut,
		handles_map_generation::{AgentType, InteractiveType, MapPrefabs, PrefabType, SetPrefab},
	},
	zyheeda_commands::ZyheedaEntityCommands,
};

#[derive(SystemParam, Debug)]
pub struct SetAgentPrefab<'w> {
	agent_prefabs: ResMut<'w, PrefabRegister<AgentType>>,
	interactive_prefabs: ResMut<'w, PrefabRegister<InteractiveType>>,
}

impl<T> SetPrefab<T> for &mut PrefabRegister<T>
where
	T: PrefabType,
{
	fn set_map_agent_prefab(&mut self, prefab: fn(ZyheedaEntityCommands, T::TTranslation, T)) {
		**self = PrefabRegister(prefab);
	}
}

impl GetContextMut<MapPrefabs<AgentType>> for SetAgentPrefab<'static> {
	type TContext<'ctx> = &'ctx mut PrefabRegister<AgentType>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SetAgentPrefab,
		_: MapPrefabs<AgentType>,
	) -> Option<Self::TContext<'ctx>> {
		Some(param.agent_prefabs.as_mut())
	}
}

impl GetContextMut<MapPrefabs<InteractiveType>> for SetAgentPrefab<'static> {
	type TContext<'ctx> = &'ctx mut PrefabRegister<InteractiveType>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SetAgentPrefab,
		_: MapPrefabs<InteractiveType>,
	) -> Option<Self::TContext<'ctx>> {
		Some(param.interactive_prefabs.as_mut())
	}
}
