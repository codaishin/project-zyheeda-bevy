use crate::{
	assets::agent_config::{AgentConfigAsset, Loadout},
	components::agent_config::{AgentConfig, InsertAgentDefaultLoadout},
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	tools::{action_key::slot::SlotKey, inventory_key::InventoryKey},
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
use std::{iter::Enumerate, slice::Iter};

impl InsertAgentDefaultLoadout {
	pub(crate) fn execute<TLoadout>(
		mut loadout: StaticSystemParam<TLoadout>,
		mut commands: ZyheedaCommands,
		agents: Query<(Entity, &AgentConfig), With<Self>>,
		configs: Res<Assets<AgentConfigAsset>>,
	) where
		TLoadout: for<'c> GetContextMut<NotLoadedOut, TContext<'c>: InsertDefaultLoadout>,
	{
		for (entity, AgentConfig { config_handle }) in agents {
			let key = NotLoadedOut { entity };
			let Some(config) = configs.get(config_handle) else {
				continue;
			};
			let Some(mut ctx) = TLoadout::get_context_mut(&mut loadout, key) else {
				continue;
			};

			ctx.insert_default_loadout(&config.loadout);
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<Self>();
			});
		}
	}
}

pub struct LoadoutIterator<'a> {
	inventory: Enumerate<Iter<'a, Option<ItemName>>>,
	slots: Iter<'a, (SlotKey, Option<ItemName>)>,
}

impl LoadoutIterator<'_> {
	fn next_inventory_item(&mut self) -> Option<(LoadoutKey, Option<ItemName>)> {
		self.inventory
			.next()
			.map(|(key, item)| (LoadoutKey::from(InventoryKey(key)), item.clone()))
	}

	fn next_slot_item(&mut self) -> Option<(LoadoutKey, Option<ItemName>)> {
		self.slots
			.next()
			.map(|(key, item)| (LoadoutKey::from(*key), item.clone()))
	}
}

impl Iterator for LoadoutIterator<'_> {
	type Item = (LoadoutKey, Option<ItemName>);

	fn next(&mut self) -> Option<Self::Item> {
		self.next_inventory_item().or_else(|| self.next_slot_item())
	}
}

impl<'a> IntoIterator for &'a Loadout {
	type Item = (LoadoutKey, Option<ItemName>);
	type IntoIter = LoadoutIterator<'a>;

	fn into_iter(self) -> LoadoutIterator<'a> {
		LoadoutIterator {
			inventory: self.inventory.iter().enumerate(),
			slots: self.slots.iter(),
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::assets::agent_config::Loadout;
	use common::tools::{action_key::slot::SlotKey, inventory_key::InventoryKey};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component, Debug, PartialEq, Default)]
	struct _LoadoutHandler {
		loadout: Vec<(LoadoutKey, Option<ItemName>)>,
	}

	impl InsertDefaultLoadout for _LoadoutHandler {
		fn insert_default_loadout<TItems>(&mut self, items: TItems)
		where
			TItems: IntoIterator<Item = (LoadoutKey, Option<ItemName>)>,
		{
			self.loadout = items.into_iter().collect()
		}
	}

	fn setup<const N: usize>(configs: [(&Handle<AgentConfigAsset>, AgentConfigAsset); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut config_assets = Assets::default();

		for (id, config) in configs {
			_ = config_assets.insert(id, config);
		}

		app.insert_resource(config_assets);
		app.add_systems(
			Update,
			InsertAgentDefaultLoadout::execute::<Query<&mut _LoadoutHandler>>,
		);

		app
	}

	#[test]
	fn insert_default_loadout() {
		let config_handle = new_handle();
		let config = AgentConfigAsset {
			loadout: Loadout {
				inventory: vec![Some(ItemName::from("inventory.item"))],
				slots: vec![(SlotKey(42), Some(ItemName::from("slot.item")))],
			},
			..default()
		};
		let mut app = setup([(&config_handle, config)]);
		let entity = app
			.world_mut()
			.spawn((AgentConfig { config_handle }, _LoadoutHandler::default()))
			.id();

		app.update();

		assert_eq!(
			Some(&_LoadoutHandler {
				loadout: vec![
					(
						LoadoutKey::Inventory(InventoryKey(0)),
						Some(ItemName::from("inventory.item"))
					),
					(
						LoadoutKey::Slot(SlotKey(42)),
						Some(ItemName::from("slot.item"))
					),
				],
			}),
			app.world().entity(entity).get::<_LoadoutHandler>(),
		);
	}

	#[test]
	fn act_only_once() {
		let config_handle = new_handle();
		let mut app = setup([(&config_handle, AgentConfigAsset::default())]);
		let entity = app
			.world_mut()
			.spawn((AgentConfig { config_handle }, _LoadoutHandler::default()))
			.id();

		app.update();
		let mut entity_ref = app.world_mut().entity_mut(entity);
		let mut handler = entity_ref.get_mut::<_LoadoutHandler>().unwrap();
		handler.loadout.clear();
		app.update();

		assert_eq!(
			Some(&_LoadoutHandler { loadout: vec![] }),
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
				AgentConfig {
					config_handle: config_handle.clone(),
				},
				_LoadoutHandler::default(),
			))
			.id();

		app.update();
		let mut configs = app.world_mut().resource_mut::<Assets<AgentConfigAsset>>();
		_ = configs.insert(
			&config_handle,
			AgentConfigAsset {
				loadout: Loadout {
					inventory: vec![Some(ItemName::from("inventory.item"))],
					slots: vec![(SlotKey(42), Some(ItemName::from("slot.item")))],
				},
				..default()
			},
		);
		app.update();

		assert_eq!(
			Some(&_LoadoutHandler {
				loadout: vec![
					(
						LoadoutKey::Inventory(InventoryKey(0)),
						Some(ItemName::from("inventory.item"))
					),
					(
						LoadoutKey::Slot(SlotKey(42)),
						Some(ItemName::from("slot.item"))
					),
				],
			}),
			app.world().entity(entity).get::<_LoadoutHandler>(),
		);
	}
}
