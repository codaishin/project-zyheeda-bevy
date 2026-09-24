use crate::resources::agents::prefab::PrefabRegister;
use bevy::{
	ecs::system::{SystemParam, SystemParamItem},
	prelude::*,
};
use common::prelude::*;

#[derive(SystemParam, Debug)]
pub struct SetAgentPrefab<'w> {
	agent_prefabs: ResMut<'w, PrefabRegister<AgentType>>,
	interactive_prefabs: ResMut<'w, PrefabRegister<InteractiveType>>,
	light_prefabs: ResMut<'w, PrefabRegister<LightType>>,
}

impl<T> SetPrefab<T> for &mut PrefabRegister<T>
where
	T: PrefabType,
{
	fn set_prefab(&mut self, prefab: fn(ZyheedaEntityCommands, T::TTransform, T)) {
		**self = PrefabRegister(prefab);
	}
}

impl GetContextMut<MapPrefabs<AgentType>> for SetAgentPrefab<'static> {
	type TContext<'ctx> = &'ctx mut PrefabRegister<AgentType>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SystemParamItem<Self>,
		_: MapPrefabs<AgentType>,
	) -> Self::TContext<'ctx> {
		param.agent_prefabs.as_mut()
	}
}

impl GetContextMut<MapPrefabs<InteractiveType>> for SetAgentPrefab<'static> {
	type TContext<'ctx> = &'ctx mut PrefabRegister<InteractiveType>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SystemParamItem<Self>,
		_: MapPrefabs<InteractiveType>,
	) -> Self::TContext<'ctx> {
		param.interactive_prefabs.as_mut()
	}
}

impl GetContextMut<MapPrefabs<LightType>> for SetAgentPrefab<'static> {
	type TContext<'ctx> = &'ctx mut PrefabRegister<LightType>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SystemParamItem<Self>,
		_: MapPrefabs<LightType>,
	) -> Self::TContext<'ctx> {
		param.light_prefabs.as_mut()
	}
}
