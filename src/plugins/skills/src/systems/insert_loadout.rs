use crate::components::{inventory::Inventory, loadout::Loadout, slots::Slots};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::TryApplyOn,
		load_asset::LoadAsset,
		loadout::LoadoutConfig,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> Loadout<T>
where
	T: LoadoutConfig + Component + ThreadSafe,
{
	pub(crate) fn insert(
		trigger: Trigger<OnInsert, T>,
		agents: Query<(&T, Option<&Inventory>, Option<&Slots>)>,
		commands: ZyheedaCommands,
		assets: ResMut<AssetServer>,
	) {
		insert_internal(trigger, agents, commands, assets);
	}
}

fn insert_internal<T, TAssets>(
	trigger: Trigger<OnInsert, T>,
	agents: Query<(&T, Option<&Inventory>, Option<&Slots>)>,
	mut commands: ZyheedaCommands,
	mut assets: ResMut<TAssets>,
) where
	T: LoadoutConfig + Component + ThreadSafe,
	TAssets: Resource + LoadAsset,
{
	let entity = trigger.target();
	let Ok((agent, inventory, slots)) = agents.get(entity) else {
		return;
	};

	commands.try_apply_on(&entity, |mut e| {
		e.try_insert(Loadout::<T>::default());

		if inventory.is_none() {
			e.try_insert(Inventory::from(
				agent
					.inventory()
					.map(|path| path.map(|path| assets.load_asset(path))),
			));
		}

		if slots.is_none() {
			e.try_insert(Slots::from(
				agent
					.slots()
					.map(|(key, path)| (key, path.map(|path| assets.load_asset(path)))),
			));
		}
	});
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::item::Item;
	use bevy::asset::AssetPath;
	use common::tools::action_key::slot::SlotKey;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp, new_handle};

	#[derive(Resource, NestedMocks)]
	struct _Assets {
		mock: Mock_Assets,
	}

	#[automock]
	impl LoadAsset for _Assets {
		fn load_asset<TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
		where
			TAsset: Asset,
			TPath: Into<AssetPath<'static>> + 'static,
		{
			self.mock.load_asset(path)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Agent {
		inventory: Vec<Option<AssetPath<'static>>>,
		slots: Vec<(SlotKey, Option<AssetPath<'static>>)>,
	}

	impl LoadoutConfig for _Agent {
		fn inventory(&self) -> impl Iterator<Item = Option<AssetPath<'static>>> {
			self.inventory.iter().cloned()
		}

		fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<AssetPath<'static>>)> {
			self.slots.iter().cloned()
		}
	}

	fn setup(assets: _Assets) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(assets);
		app.add_observer(insert_internal::<_Agent, _Assets>);

		app
	}

	#[test]
	fn insert_loadout_components() {
		let inventory_item = new_handle();
		let slot_item = new_handle();
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Item, AssetPath<'static>>()
				.with(eq(AssetPath::from("inventory item")))
				.return_const(inventory_item.clone());
			mock.expect_load_asset::<Item, AssetPath<'static>>()
				.with(eq(AssetPath::from("slot item")))
				.return_const(slot_item.clone());
		}));

		let entity = app.world_mut().spawn(_Agent {
			inventory: vec![Some(AssetPath::from("inventory item"))],
			slots: vec![(SlotKey(42), Some(AssetPath::from("slot item")))],
		});

		assert_eq!(
			(
				Some(&Loadout::<_Agent>::default()),
				Some(&Inventory::from([Some(inventory_item)])),
				Some(&Slots::from([(SlotKey(42), Some(slot_item))]))
			),
			(
				entity.get::<Loadout<_Agent>>(),
				entity.get::<Inventory>(),
				entity.get::<Slots>(),
			)
		);
	}

	#[test]
	fn no_inventory_mapping_if_inventory_already_present() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Item, AssetPath<'static>>()
				.with(eq(AssetPath::from("inventory item")))
				.never();
			mock.expect_load_asset::<Item, AssetPath<'static>>()
				.with(eq(AssetPath::from("slot item")))
				.return_const(new_handle());
		}));

		app.world_mut().spawn((
			_Agent {
				inventory: vec![Some(AssetPath::from("inventory item"))],
				slots: vec![(SlotKey(42), Some(AssetPath::from("slot item")))],
			},
			Inventory::from([]),
		));
	}

	#[test]
	fn insert_loadout_if_inventory_already_present() {
		let slot_item = new_handle();
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Item, AssetPath<'static>>()
				.with(eq(AssetPath::from("inventory item")))
				.return_const(new_handle());
			mock.expect_load_asset::<Item, AssetPath<'static>>()
				.with(eq(AssetPath::from("slot item")))
				.return_const(slot_item.clone());
		}));

		let entity = app.world_mut().spawn((
			_Agent {
				inventory: vec![Some(AssetPath::from("inventory item"))],
				slots: vec![(SlotKey(42), Some(AssetPath::from("slot item")))],
			},
			Inventory::from([]),
		));

		assert_eq!(
			(
				Some(&Loadout::<_Agent>::default()),
				Some(&Inventory::from([])),
				Some(&Slots::from([(SlotKey(42), Some(slot_item))]))
			),
			(
				entity.get::<Loadout<_Agent>>(),
				entity.get::<Inventory>(),
				entity.get::<Slots>(),
			)
		);
	}

	#[test]
	fn no_slot_mapping_if_slots_already_present() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Item, AssetPath<'static>>()
				.with(eq(AssetPath::from("inventory item")))
				.return_const(new_handle());
			mock.expect_load_asset::<Item, AssetPath<'static>>()
				.with(eq(AssetPath::from("slot item")))
				.never();
		}));

		app.world_mut().spawn((
			_Agent {
				inventory: vec![Some(AssetPath::from("inventory item"))],
				slots: vec![(SlotKey(42), Some(AssetPath::from("slot item")))],
			},
			Slots::from([]),
		));
	}

	#[test]
	fn insert_loadout_if_slots_already_present() {
		let inventory_item = new_handle();
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Item, AssetPath<'static>>()
				.with(eq(AssetPath::from("inventory item")))
				.return_const(inventory_item.clone());
			mock.expect_load_asset::<Item, AssetPath<'static>>()
				.with(eq(AssetPath::from("slot item")))
				.return_const(new_handle());
		}));

		let entity = app.world_mut().spawn((
			_Agent {
				inventory: vec![Some(AssetPath::from("inventory item"))],
				slots: vec![(SlotKey(42), Some(AssetPath::from("slot item")))],
			},
			Slots::from([]),
		));

		assert_eq!(
			(
				Some(&Loadout::<_Agent>::default()),
				Some(&Inventory::from([Some(inventory_item)])),
				Some(&Slots::from([]))
			),
			(
				entity.get::<Loadout<_Agent>>(),
				entity.get::<Inventory>(),
				entity.get::<Slots>(),
			)
		);
	}

	#[test]
	fn insert_loadout_if_slots_and_inventory_already_present() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Item, AssetPath<'static>>()
				.with(eq(AssetPath::from("inventory item")))
				.return_const(new_handle());
			mock.expect_load_asset::<Item, AssetPath<'static>>()
				.with(eq(AssetPath::from("slot item")))
				.return_const(new_handle());
		}));

		let entity = app.world_mut().spawn((
			_Agent {
				inventory: vec![Some(AssetPath::from("inventory item"))],
				slots: vec![(SlotKey(42), Some(AssetPath::from("slot item")))],
			},
			Inventory::from([]),
			Slots::from([]),
		));

		assert_eq!(
			(
				Some(&Loadout::<_Agent>::default()),
				Some(&Inventory::from([])),
				Some(&Slots::from([]))
			),
			(
				entity.get::<Loadout<_Agent>>(),
				entity.get::<Inventory>(),
				entity.get::<Slots>(),
			)
		);
	}
}
