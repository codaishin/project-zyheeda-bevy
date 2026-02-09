use crate::resources::agents::prefab::AgentPrefab;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::GetContextMut,
		handles_map_generation::{
			AgentPrefab as AgentPrefabMarker,
			AgentType,
			GroundPosition,
			SetMapAgentPrefab,
		},
	},
	zyheeda_commands::ZyheedaEntityCommands,
};

#[derive(SystemParam, Debug)]
pub struct SetAgentPrefab<'w> {
	prefab: ResMut<'w, AgentPrefab>,
}

impl SetMapAgentPrefab for &mut AgentPrefab {
	fn set_map_agent_prefab(
		&mut self,
		prefab: fn(ZyheedaEntityCommands, GroundPosition, AgentType),
	) {
		**self = AgentPrefab(prefab);
	}
}

impl GetContextMut<AgentPrefabMarker> for SetAgentPrefab<'_> {
	type TContext<'ctx> = &'ctx mut AgentPrefab;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SetAgentPrefab,
		_: AgentPrefabMarker,
	) -> Option<Self::TContext<'ctx>> {
		Some(param.prefab.as_mut())
	}
}
