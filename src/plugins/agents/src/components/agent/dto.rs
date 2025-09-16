use crate::components::agent::Agent;
use bevy::asset::{AssetPath, Handle};
use common::{
	errors::Unreachable,
	traits::{handles_custom_assets::TryLoadFrom, load_asset::LoadAsset},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AgentDto {
	#[serde(skip_serializing_if = "Option::is_none")]
	asset_path: Option<String>,
}

impl From<Agent> for AgentDto {
	fn from(agent: Agent) -> Self {
		match agent {
			Agent::Path(asset_path) => Self {
				asset_path: Some(asset_path.to_string()),
			},
			Agent::Loading(handle) => Self {
				asset_path: handle.path().map(AssetPath::to_string),
			},
			Agent::Loaded(handle) => Self {
				asset_path: handle.path().map(AssetPath::to_string),
			},
		}
	}
}

impl TryLoadFrom<AgentDto> for Agent {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		AgentDto { asset_path }: AgentDto,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		Ok(match asset_path {
			Some(path) => Self::Loading(asset_server.load_asset(path)),
			None => Self::Loading(Handle::default()), // FIXME: do we need a default asset model, just in case?
		})
	}
}
