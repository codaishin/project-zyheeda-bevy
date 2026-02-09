use crate::{
	components::{inventory::Inventory, loadout::Loadout, slots::Slots},
	system_parameters::loadout::LoadoutPrep,
};
use bevy::prelude::*;
use common::{
	tools::inventory_key::InventoryKey,
	traits::{
		accessors::get::{GetContextMut, GetMut, TryApplyOn},
		handles_loadout::{
			LoadoutKey,
			insert_default_loadout::{InsertDefaultLoadout, NotLoadedOut},
		},
		load_asset::LoadAsset,
		loadout::ItemName,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

impl InsertDefaultLoadout for DefaultLoadout<'_> {
	fn insert_default_loadout<TItems>(&mut self, loadout: TItems)
	where
		TItems: IntoIterator<Item = (LoadoutKey, Option<ItemName>)>,
	{
		let loadout = loadout.into_iter().collect();
		self.entity
			.trigger(move |entity| InsertLoadoutEvent { entity, loadout });
	}
}

pub struct DefaultLoadout<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
}

impl DefaultLoadout<'_> {
	fn asset_path(name: &ItemName) -> String {
		format!("items/{name}.item")
	}

	pub(crate) fn insert<TAssetServer>(
		on_insert_loadout: On<InsertLoadoutEvent>,
		mut commands: ZyheedaCommands,
		mut server: ResMut<TAssetServer>,
	) where
		TAssetServer: Resource + LoadAsset,
	{
		let mut slots = Slots::default();
		let mut inventory = Inventory::default();

		for (key, name) in &on_insert_loadout.loadout {
			let item = name
				.as_ref()
				.map(|name| server.load_asset(Self::asset_path(name)));

			match key {
				LoadoutKey::Inventory(InventoryKey(key)) => {
					inventory.fill_up_to(*key);
					inventory.0[*key] = item;
				}
				LoadoutKey::Slot(key) => {
					slots.items.insert(*key, item);
				}
			}
		}

		commands.try_apply_on(&on_insert_loadout.entity, move |mut e| {
			e.try_insert_if_new((Loadout, inventory, slots));
		});
	}
}

impl GetContextMut<NotLoadedOut> for LoadoutPrep<'_, '_> {
	type TContext<'ctx> = DefaultLoadout<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut LoadoutPrep,
		NotLoadedOut { entity }: NotLoadedOut,
	) -> Option<Self::TContext<'ctx>> {
		if param.inventories.contains(entity) && param.slots.contains(entity) {
			return None;
		}

		let entity = param.commands.get_mut(&entity)?;

		Some(DefaultLoadout { entity })
	}
}

#[derive(EntityEvent, Debug, PartialEq)]
pub(crate) struct InsertLoadoutEvent {
	entity: Entity,
	loadout: Vec<(LoadoutKey, Option<ItemName>)>,
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{components::loadout::Loadout, item::Item, skills::Skill};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::{
		tools::{action_key::slot::SlotKey, inventory_key::InventoryKey},
		traits::load_asset::mock::MockAssetServer,
	};
	use testing::{SingleThreadedApp, new_handle};

	fn setup(server: MockAssetServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<Assets<Skill>>();
		app.insert_resource(server);
		app.add_observer(DefaultLoadout::insert::<MockAssetServer>);

		app
	}

	#[test]
	fn insert_inventory_and_slots() -> Result<(), RunSystemError> {
		let handles = [
			new_handle::<Item>(),
			new_handle::<Item>(),
			new_handle::<Item>(),
			new_handle::<Item>(),
		];
		let mut app = setup(
			MockAssetServer::default()
				.path("items/item_0.item")
				.returns(handles[0].clone())
				.path("items/item_1.item")
				.returns(handles[1].clone())
				.path("items/item_2.item")
				.returns(handles[2].clone())
				.path("items/item_3.item")
				.returns(handles[3].clone()),
		);
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut loadout: LoadoutPrep| {
				let key = NotLoadedOut { entity };
				let mut ctx = LoadoutPrep::get_context_mut(&mut loadout, key).unwrap();
				ctx.insert_default_loadout([
					(
						LoadoutKey::from(InventoryKey(0)),
						Some(ItemName::from("item_0")),
					),
					(
						LoadoutKey::from(InventoryKey(1)),
						Some(ItemName::from("item_1")),
					),
					(LoadoutKey::from(SlotKey(0)), Some(ItemName::from("item_2"))),
					(LoadoutKey::from(SlotKey(1)), Some(ItemName::from("item_3"))),
				]);
			})?;

		assert_eq!(
			(
				Some(&Inventory(vec![
					Some(handles[0].clone()),
					Some(handles[1].clone())
				])),
				Some(&Slots::from([
					(SlotKey(0), Some(handles[2].clone())),
					(SlotKey(1), Some(handles[3].clone()))
				]))
			),
			(
				app.world().entity(entity).get::<Inventory>(),
				app.world().entity(entity).get::<Slots>(),
			)
		);
		Ok(())
	}

	#[test]
	fn insert_inventory_non_continuously() -> Result<(), RunSystemError> {
		let handles = [new_handle::<Item>(), new_handle::<Item>()];
		let mut app = setup(
			MockAssetServer::default()
				.path("items/item_0.item")
				.returns(handles[0].clone())
				.path("items/item_1.item")
				.returns(handles[1].clone()),
		);
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut loadout: LoadoutPrep| {
				let key = NotLoadedOut { entity };
				let mut ctx = LoadoutPrep::get_context_mut(&mut loadout, key).unwrap();
				ctx.insert_default_loadout([
					(
						LoadoutKey::from(InventoryKey(3)),
						Some(ItemName::from("item_0")),
					),
					(
						LoadoutKey::from(InventoryKey(1)),
						Some(ItemName::from("item_1")),
					),
				]);
			})?;

		assert_eq!(
			Some(&Inventory(vec![
				None,
				Some(handles[1].clone()),
				None,
				Some(handles[0].clone()),
			])),
			app.world().entity(entity).get::<Inventory>()
		);
		Ok(())
	}

	#[test]
	fn no_context_if_slots_and_inventory_set() -> Result<(), RunSystemError> {
		let mut app = setup(MockAssetServer::default());
		let entity = app
			.world_mut()
			.spawn((Slots::default(), Inventory::default()))
			.id();

		let ctx_is_none = app
			.world_mut()
			.run_system_once(move |mut loadout: LoadoutPrep| {
				LoadoutPrep::get_context_mut(&mut loadout, NotLoadedOut { entity }).is_none()
			})?;

		assert!(ctx_is_none);
		Ok(())
	}

	#[test]
	fn only_set_inventory_when_only_inventory_missing() -> Result<(), RunSystemError> {
		let handles = [
			new_handle::<Item>(),
			new_handle::<Item>(),
			new_handle::<Item>(),
			new_handle::<Item>(),
			new_handle::<Item>(),
		];
		let mut app = setup(
			MockAssetServer::default()
				.path("items/item_0.item")
				.returns(handles[0].clone())
				.path("items/item_1.item")
				.returns(handles[1].clone())
				.path("items/item_2.item")
				.returns(handles[2].clone())
				.path("items/item_3.item")
				.returns(handles[3].clone())
				.path("items/item_4.item")
				.returns(handles[4].clone()),
		);
		let entity = app
			.world_mut()
			.spawn(Slots::from([
				(SlotKey(0), Some(handles[0].clone())),
				(SlotKey(1), Some(handles[1].clone())),
			]))
			.id();

		app.world_mut()
			.run_system_once(move |mut loadout: LoadoutPrep| {
				let key = NotLoadedOut { entity };
				let mut ctx = LoadoutPrep::get_context_mut(&mut loadout, key).unwrap();
				ctx.insert_default_loadout([
					(LoadoutKey::from(SlotKey(2)), Some(ItemName::from("item_2"))),
					(LoadoutKey::from(SlotKey(3)), Some(ItemName::from("item_3"))),
					(
						LoadoutKey::from(InventoryKey(0)),
						Some(ItemName::from("item_4")),
					),
				]);
			})?;

		assert_eq!(
			(
				Some(&Inventory(vec![Some(handles[4].clone())])),
				Some(&Slots::from([
					(SlotKey(0), Some(handles[0].clone())),
					(SlotKey(1), Some(handles[1].clone())),
				]))
			),
			(
				app.world().entity(entity).get::<Inventory>(),
				app.world().entity(entity).get::<Slots>(),
			)
		);
		Ok(())
	}

	#[test]
	fn only_set_slots_when_only_slots_missing() -> Result<(), RunSystemError> {
		let handles = [
			new_handle::<Item>(),
			new_handle::<Item>(),
			new_handle::<Item>(),
			new_handle::<Item>(),
			new_handle::<Item>(),
		];
		let mut app = setup(
			MockAssetServer::default()
				.path("items/item_0.item")
				.returns(handles[0].clone())
				.path("items/item_1.item")
				.returns(handles[1].clone())
				.path("items/item_2.item")
				.returns(handles[2].clone())
				.path("items/item_3.item")
				.returns(handles[3].clone())
				.path("items/item_4.item")
				.returns(handles[4].clone()),
		);
		let entity = app
			.world_mut()
			.spawn(Inventory(vec![
				Some(handles[0].clone()),
				Some(handles[1].clone()),
			]))
			.id();

		app.world_mut()
			.run_system_once(move |mut loadout: LoadoutPrep| {
				let key = NotLoadedOut { entity };
				let mut ctx = LoadoutPrep::get_context_mut(&mut loadout, key).unwrap();
				ctx.insert_default_loadout([
					(
						LoadoutKey::from(InventoryKey(2)),
						Some(ItemName::from("item_2")),
					),
					(
						LoadoutKey::from(InventoryKey(3)),
						Some(ItemName::from("item_3")),
					),
					(LoadoutKey::from(SlotKey(4)), Some(ItemName::from("item_4"))),
				]);
			})?;

		assert_eq!(
			(
				Some(&Slots::from([(SlotKey(4), Some(handles[4].clone()))])),
				Some(&Inventory(vec![
					Some(handles[0].clone()),
					Some(handles[1].clone()),
				]))
			),
			(
				app.world().entity(entity).get::<Slots>(),
				app.world().entity(entity).get::<Inventory>(),
			)
		);
		Ok(())
	}

	#[test]
	fn insert_loadout_component() -> Result<(), RunSystemError> {
		let mut app = setup(MockAssetServer::default());
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut loadout: LoadoutPrep| {
				let key = NotLoadedOut { entity };
				let mut ctx = LoadoutPrep::get_context_mut(&mut loadout, key).unwrap();
				ctx.insert_default_loadout([]);
			})?;

		assert!(app.world().entity(entity).contains::<Loadout>());
		Ok(())
	}
}
