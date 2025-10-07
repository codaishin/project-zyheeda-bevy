pub(crate) mod tag;

use crate::{
	assets::agent_config::{AgentConfigAsset, AgentConfigData},
	components::{actions::Actions, agent::tag::AgentTag},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::{GravityScale, RigidBody};
use common::{
	components::{
		collider_relationship::InteractionTarget,
		is_blocker::{Blocker, IsBlocker},
		persistent_entity::PersistentEntity,
	},
	traits::{
		accessors::get::GetFromSystemParam,
		handles_agents::{AgentActionTarget, AgentConfig, AgentType, CurrentAction, Spawn},
	},
	zyheeda_commands::ZyheedaCommands,
};

#[derive(Component, Clone, Debug, PartialEq)]
#[require(
	InteractionTarget,
	PersistentEntity,
	Transform,
	Visibility,
	RigidBody = RigidBody::Dynamic,
	GravityScale = GravityScale(0.),
	IsBlocker = [Blocker::Character],
)]
pub struct Agent<TAsset = AgentConfigAsset>
where
	TAsset: Asset,
{
	pub(crate) agent_type: AgentType,
	pub(crate) config_handle: Handle<TAsset>,
	pub(crate) entity: Entity,
}

impl Spawn for Agent {
	fn spawn<'a>(commands: &'a mut ZyheedaCommands, agent_type: AgentType) -> EntityCommands<'a> {
		commands.spawn(AgentTag(agent_type))
	}
}

impl<TAsset> GetFromSystemParam<AgentConfig> for Agent<TAsset>
where
	TAsset: Asset + Clone,
{
	type TParam<'w, 's> = Res<'w, Assets<TAsset>>;
	type TItem<'i> = AgentConfigData<'i, TAsset>;

	fn get_from_param<'a>(
		&'a self,
		_: &AgentConfig,
		assets: &'a Res<Assets<TAsset>>,
	) -> Option<Self::TItem<'a>> {
		assets
			.get(&self.config_handle)
			.map(|asset| AgentConfigData {
				agent_type: self.agent_type,
				asset,
			})
	}
}

impl<TAsset> GetFromSystemParam<CurrentAction> for Agent<TAsset>
where
	TAsset: Asset,
{
	type TParam<'world, 'state> = Query<'world, 'state, &'static Actions>;
	type TItem<'item> = AgentActionTarget;

	fn get_from_param(
		&self,
		key: &CurrentAction,
		actions: &Query<&Actions>,
	) -> Option<AgentActionTarget> {
		let Actions(actions) = actions.get(self.entity).ok()?;
		actions.get(key).copied()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{enemy::void_sphere::VoidSphere, player::Player};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use std::{collections::HashMap, sync::LazyLock};
	use testing::SingleThreadedApp;

	static HANDLE: LazyLock<Handle<_Asset>> = LazyLock::new(Handle::default);

	#[derive(Asset, TypePath, Debug, PartialEq, Clone)]
	struct _Asset;

	mod config {
		use super::*;
		use test_case::test_case;

		fn setup() -> App {
			let mut app = App::new().single_threaded(Update);
			let mut assets = Assets::default();

			assets.insert(&*HANDLE, _Asset);
			app.insert_resource(assets);

			app
		}

		#[test_case(AgentType::from(Player))]
		#[test_case(AgentType::from(VoidSphere))]
		fn get_some_data_when_handle_set(agent_type: AgentType) -> Result<(), RunSystemError> {
			let mut app = setup();
			let agent = Agent {
				agent_type,
				config_handle: HANDLE.clone(),
				entity: Entity::from_raw(123),
			};

			app.world_mut()
				.run_system_once(move |assets: Res<Assets<_Asset>>| {
					assert_eq!(
						Some(AgentConfigData {
							agent_type,
							asset: &_Asset,
						}),
						agent.get_from_param(&AgentConfig, &assets)
					);
				})
		}
	}

	mod action {
		use super::*;
		use test_case::test_case;

		fn setup() -> App {
			App::new().single_threaded(Update)
		}

		#[test_case(CurrentAction::Movement, AgentActionTarget::Direction(Dir3::Z); "movement")]
		#[test_case(CurrentAction::UseSkill, AgentActionTarget::Point(Vec3::new(1., 2.,3.)); "skill")]
		fn get_current(
			key: CurrentAction,
			target: AgentActionTarget,
		) -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(Actions(HashMap::from([(key, target)])))
				.id();
			let agent = Agent {
				agent_type: AgentType::Player,
				config_handle: HANDLE.clone(),
				entity,
			};

			app.world_mut()
				.run_system_once(move |actions: Query<&'static Actions>| {
					assert_eq!(Some(target), agent.get_from_param(&key, &actions));
				})
		}
	}
}
