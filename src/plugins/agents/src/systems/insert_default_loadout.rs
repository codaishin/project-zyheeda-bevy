use crate::components::{agent::Agent, insert_agent_default_loadout::InsertAgentDefaultLoadout};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, TryApplyOn},
		handles_loadout::{
			LoadoutKey,
			insert_default_loadout::{InsertDefaultLoadout, NotLoadedOut},
		},
		loadout::ItemName,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl InsertAgentDefaultLoadout {
	pub(crate) fn execute<TConfig, TLoadout>(
		mut loadout: StaticSystemParam<TLoadout>,
		mut commands: ZyheedaCommands,
		agents: Query<(Entity, &Agent<TConfig>), With<Self>>,
		configs: Res<Assets<TConfig>>,
	) where
		TLoadout: for<'c> GetContextMut<NotLoadedOut, TContext<'c>: InsertDefaultLoadout>,
		TConfig: Asset + internal::GetDefaultLoadout,
	{
		for (entity, Agent { config_handle, .. }) in agents {
			let key = NotLoadedOut { entity };
			let Some(config) = configs.get(config_handle) else {
				continue;
			};
			let Some(mut ctx) = TLoadout::get_context_mut(&mut loadout, key) else {
				continue;
			};

			ctx.insert_default_loadout(config.get_default_loadout());
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<Self>();
			});
		}
	}
}

pub(crate) mod internal {
	use super::*;

	pub trait GetDefaultLoadout {
		type TLoadout<'a>: IntoIterator<Item = (LoadoutKey, Option<ItemName>)>
		where
			Self: 'a;

		fn get_default_loadout(&self) -> Self::TLoadout<'_>;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_map_generation::AgentType;
	use std::{any::type_name, collections::HashMap};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component, Debug, PartialEq, Default)]
	struct _LoadoutHandler {
		called_with: HashMap<&'static str, usize>,
	}

	impl InsertDefaultLoadout for _LoadoutHandler {
		fn insert_default_loadout<TItems>(&mut self, _: TItems)
		where
			TItems: IntoIterator<Item = (LoadoutKey, Option<ItemName>)>,
		{
			// `TypeId` and `mockall` not working, because of missing `'static` constraint
			// so we count calls via the type name
			*self.called_with.entry(type_name::<TItems>()).or_default() += 1;
		}
	}

	#[derive(Asset, TypePath)]
	struct _Config;

	impl internal::GetDefaultLoadout for _Config {
		type TLoadout<'a> = _ConfigIter;

		fn get_default_loadout(&self) -> Self::TLoadout<'_> {
			_ConfigIter
		}
	}

	struct _ConfigIter;

	impl Iterator for _ConfigIter {
		type Item = (LoadoutKey, Option<ItemName>);

		fn next(&mut self) -> Option<Self::Item> {
			None
		}
	}

	fn setup<const N: usize>(handles: [&Handle<_Config>; N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut configs = Assets::default();

		for id in handles {
			configs.insert(id, _Config);
		}

		app.insert_resource(configs);
		app.add_systems(
			Update,
			InsertAgentDefaultLoadout::execute::<_Config, Query<&mut _LoadoutHandler>>,
		);

		app
	}

	#[test]
	fn insert_default_loadout() {
		let config_handle = new_handle();
		let mut app = setup([&config_handle]);
		let entity = app
			.world_mut()
			.spawn((
				Agent {
					agent_type: AgentType::Player,
					config_handle,
				},
				_LoadoutHandler::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_LoadoutHandler {
				called_with: HashMap::from([(type_name::<_ConfigIter>(), 1)])
			}),
			app.world().entity(entity).get::<_LoadoutHandler>(),
		);
	}

	#[test]
	fn act_only_once() {
		let config_handle = new_handle();
		let mut app = setup([&config_handle]);
		let entity = app
			.world_mut()
			.spawn((
				Agent {
					agent_type: AgentType::Player,
					config_handle,
				},
				_LoadoutHandler::default(),
			))
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&_LoadoutHandler {
				called_with: HashMap::from([(type_name::<_ConfigIter>(), 1)])
			}),
			app.world().entity(entity).get::<_LoadoutHandler>(),
		);
	}

	#[test]
	fn insert_default_loadout_when_asset_available_later() {
		let config_handle = new_handle();
		let mut app = setup([]);
		let entity = app
			.world_mut()
			.spawn((
				Agent {
					agent_type: AgentType::Player,
					config_handle: config_handle.clone(),
				},
				_LoadoutHandler::default(),
			))
			.id();

		app.update();
		let mut configs = app.world_mut().resource_mut::<Assets<_Config>>();
		configs.insert(&config_handle, _Config);
		app.update();

		assert_eq!(
			Some(&_LoadoutHandler {
				called_with: HashMap::from([(type_name::<_ConfigIter>(), 1)])
			}),
			app.world().entity(entity).get::<_LoadoutHandler>(),
		);
	}
}
