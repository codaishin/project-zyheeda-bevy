use crate::{
	components::{inventory::Inventory, loadout::Loadout, slots::Slots},
	system_parameters::loadout::LoadoutPrep,
};
use bevy::prelude::*;
use common::{
	tools::inventory_key::InventoryKey,
	traits::{
		accessors::get::{GetContextMut, GetMut},
		handles_loadout::{
			LoadoutKey,
			default_items::{InsertDefaultLoadout, NotLoadedOut},
		},
		load_asset::LoadAsset,
		loadout::ItemName,
	},
	zyheeda_commands::ZyheedaEntityCommands,
};

impl<TAssetServer> InsertDefaultLoadout for PrepareLoadout<'_, TAssetServer>
where
	TAssetServer: Resource + LoadAsset,
{
	fn insert_default_loadout<TItems>(&mut self, loadout: TItems)
	where
		TItems: IntoIterator<Item = (LoadoutKey, ItemName)>,
	{
		let mut slots = Slots::default();
		let mut inventory = Inventory::default();

		for (key, name) in loadout.into_iter() {
			let item = self.server.load_asset(asset_path(name));
			match key {
				LoadoutKey::Inventory(InventoryKey(key)) => {
					inventory.fill_up_to(key);
					inventory.0[key] = Some(item);
				}
				LoadoutKey::Slot(key) => {
					slots.items.insert(key, Some(item));
				}
			}
		}

		self.entity
			.try_insert_if_new((Loadout::<()>::default(), inventory, slots));
	}
}

fn asset_path(name: ItemName) -> String {
	format!("items/{name}.item")
}

pub struct PrepareLoadout<'ctx, TAssetServer>
where
	TAssetServer: Resource + LoadAsset,
{
	entity: ZyheedaEntityCommands<'ctx>,
	server: &'ctx mut TAssetServer,
}

impl<TAssetServer> GetContextMut<NotLoadedOut> for LoadoutPrep<'_, '_, TAssetServer>
where
	TAssetServer: Resource + LoadAsset,
{
	type TContext<'ctx> = PrepareLoadout<'ctx, TAssetServer>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut LoadoutPrep<TAssetServer>,
		NotLoadedOut { entity }: NotLoadedOut,
	) -> Option<Self::TContext<'ctx>> {
		if param.inventories.contains(entity) && param.slots.contains(entity) {
			return None;
		}

		let entity = param.commands.get_mut(&entity)?;
		let server = param.asset_server.as_mut();

		Some(PrepareLoadout { entity, server })
	}
}

#[cfg(test)]
mod tests {
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
			.run_system_once(move |mut loadout: LoadoutPrep<MockAssetServer>| {
				let key = NotLoadedOut { entity };
				let mut ctx = LoadoutPrep::get_context_mut(&mut loadout, key).unwrap();
				ctx.insert_default_loadout([
					(LoadoutKey::from(InventoryKey(0)), ItemName::from("item_0")),
					(LoadoutKey::from(InventoryKey(1)), ItemName::from("item_1")),
					(LoadoutKey::from(SlotKey(0)), ItemName::from("item_2")),
					(LoadoutKey::from(SlotKey(1)), ItemName::from("item_3")),
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
			.run_system_once(move |mut loadout: LoadoutPrep<MockAssetServer>| {
				let key = NotLoadedOut { entity };
				let mut ctx = LoadoutPrep::get_context_mut(&mut loadout, key).unwrap();
				ctx.insert_default_loadout([
					(LoadoutKey::from(InventoryKey(3)), ItemName::from("item_0")),
					(LoadoutKey::from(InventoryKey(1)), ItemName::from("item_1")),
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

		let ctx_is_none =
			app.world_mut()
				.run_system_once(move |mut loadout: LoadoutPrep<MockAssetServer>| {
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
			.run_system_once(move |mut loadout: LoadoutPrep<MockAssetServer>| {
				let key = NotLoadedOut { entity };
				let mut ctx = LoadoutPrep::get_context_mut(&mut loadout, key).unwrap();
				ctx.insert_default_loadout([
					(LoadoutKey::from(SlotKey(2)), ItemName::from("item_2")),
					(LoadoutKey::from(SlotKey(3)), ItemName::from("item_3")),
					(LoadoutKey::from(InventoryKey(0)), ItemName::from("item_4")),
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
			.run_system_once(move |mut loadout: LoadoutPrep<MockAssetServer>| {
				let key = NotLoadedOut { entity };
				let mut ctx = LoadoutPrep::get_context_mut(&mut loadout, key).unwrap();
				ctx.insert_default_loadout([
					(LoadoutKey::from(InventoryKey(2)), ItemName::from("item_2")),
					(LoadoutKey::from(InventoryKey(3)), ItemName::from("item_3")),
					(LoadoutKey::from(SlotKey(4)), ItemName::from("item_4")),
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
			.run_system_once(move |mut loadout: LoadoutPrep<MockAssetServer>| {
				let key = NotLoadedOut { entity };
				let mut ctx = LoadoutPrep::get_context_mut(&mut loadout, key).unwrap();
				ctx.insert_default_loadout([]);
			})?;

		assert!(app.world().entity(entity).contains::<Loadout>());
		Ok(())
	}
}
