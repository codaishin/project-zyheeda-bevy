mod dto;

use crate::{
	assets::agent_config::{AgentConfigAsset, AgentConfigRef},
	components::agent::dto::AgentDto,
};
use bevy::{asset::AssetPath, prelude::*};
use bevy_rapier3d::prelude::{GravityScale, RigidBody};
use common::{
	components::{
		collider_relationship::InteractionTarget,
		is_blocker::{Blocker, IsBlocker},
		persistent_entity::PersistentEntity,
	},
	traits::{
		accessors::get::GetFromSystemParam,
		handles_agents::{AgentConfig, AgentType},
		handles_enemies::EnemyType,
	},
};
use macros::{SavableComponent, agent_asset};

#[derive(Component, SavableComponent, Clone, Debug, PartialEq)]
#[savable_component(dto = AgentDto)]
#[require(
	InteractionTarget,
	PersistentEntity,
	Transform,
	Visibility,
	RigidBody = RigidBody::Dynamic,
	GravityScale = GravityScale(0.),
	IsBlocker = [Blocker::Character],
)]
pub enum Agent<TAsset = AgentConfigAsset>
where
	TAsset: Asset,
{
	Path(AssetPath<'static>),
	Handle(Handle<TAsset>),
}

impl From<AgentType> for Agent {
	fn from(agent_type: AgentType) -> Self {
		Self::Path(match agent_type {
			AgentType::Player => AssetPath::from(agent_asset!("player")),
			AgentType::Enemy(EnemyType::VoidSphere) => AssetPath::from(agent_asset!("void_sphere")),
		})
	}
}

impl<TAsset> GetFromSystemParam<AgentConfig> for Agent<TAsset>
where
	TAsset: Asset + Clone,
{
	type TParam<'w, 's> = Res<'w, Assets<TAsset>>;
	type TItem<'i> = AgentConfigRef<'i, TAsset>;

	fn get_from_param<'a>(
		&'a self,
		_: &AgentConfig,
		assets: &'a Res<Assets<TAsset>>,
	) -> Option<Self::TItem<'a>> {
		match self {
			Agent::Path(..) => None,
			Agent::Handle(handle) => assets.get(handle).map(AgentConfigRef::from),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use std::sync::LazyLock;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	static HANDLE: LazyLock<Handle<_Asset>> = LazyLock::new(Handle::default);

	#[derive(Asset, TypePath, Debug, PartialEq, Clone)]
	struct _Asset;

	#[test_case(Agent::Path(AssetPath::from("my/path.agent")), None; "none for path")]
	#[test_case(Agent::Handle(HANDLE.clone()), Some(AgentConfigRef::from(&_Asset)); "some when loaded")]
	fn get_handle(
		agent: Agent<_Asset>,
		expected: Option<AgentConfigRef<'static, _Asset>>,
	) -> Result<(), RunSystemError> {
		let mut app = App::new().single_threaded(Update);
		let mut assets = Assets::default();
		assets.insert(&*HANDLE, _Asset);
		app.insert_resource(assets);

		app.world_mut()
			.run_system_once(move |assets: Res<Assets<_Asset>>| {
				assert_eq!(expected, agent.get_from_param(&AgentConfig, &assets));
			})
	}
}
