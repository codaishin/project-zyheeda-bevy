use bevy::prelude::*;
use common::{
	traits::handles_map_generation::{AgentType, GroundPosition},
	zyheeda_commands::ZyheedaEntityCommands,
};

#[derive(Resource, Debug)]
pub struct AgentPrefab(pub(crate) fn(ZyheedaEntityCommands, GroundPosition, AgentType));

impl AgentPrefab {
	const fn noop(_: ZyheedaEntityCommands, _: GroundPosition, _: AgentType) {}

	pub(crate) fn apply(
		&self,
		entity: ZyheedaEntityCommands,
		ground_position: GroundPosition,
		agent_type: AgentType,
	) {
		(self.0)(entity, ground_position, agent_type)
	}
}

impl Default for AgentPrefab {
	fn default() -> Self {
		Self(Self::noop)
	}
}

impl PartialEq for AgentPrefab {
	fn eq(&self, other: &Self) -> bool {
		std::ptr::fn_addr_eq(self.0, other.0)
	}
}
