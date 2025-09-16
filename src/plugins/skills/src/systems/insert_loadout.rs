use crate::components::{inventory::Inventory, loadout::Loadout, slots::Slots};
use bevy::prelude::*;
use common::{
	traits::{
		accessors::get::{AssociatedSystemParam, GetFromSystemParam, TryApplyOn},
		load_asset::LoadAsset,
		loadout::LoadoutConfig,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<TAgent> Loadout<TAgent>
where
	TAgent: Component + GetFromSystemParam<()>,
	for<'i> TAgent::TItem<'i>: LoadoutConfig,
{
	#[allow(clippy::type_complexity)]
	pub(crate) fn insert(
		agents: Query<
			(Entity, &TAgent, Option<&Inventory>, Option<&Slots>),
			Without<Loadout<TAgent>>,
		>,
		commands: ZyheedaCommands,
		server: ResMut<AssetServer>,
		param: AssociatedSystemParam<TAgent, ()>,
	) {
		insert_internal(agents, commands, server, param);
	}
}

#[allow(clippy::type_complexity)]
fn insert_internal<TAgent, TAssetServer>(
	agents: Query<(Entity, &TAgent, Option<&Inventory>, Option<&Slots>), Without<Loadout<TAgent>>>,
	mut commands: ZyheedaCommands,
	mut server: ResMut<TAssetServer>,
	param: AssociatedSystemParam<TAgent, ()>,
) where
	TAgent: Component + GetFromSystemParam<()>,
	TAssetServer: Resource + LoadAsset,
	for<'i> TAgent::TItem<'i>: LoadoutConfig,
{
	for (entity, agent, inventory, slots) in &agents {
		let Some(config) = agent.get_from_param(&(), &param) else {
			continue;
		};

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(Loadout::<TAgent>::default());

			if inventory.is_none() {
				e.try_insert(Inventory::from(
					config
						.inventory()
						.map(|path| path.map(|path| server.load_asset(path))),
				));
			}

			if slots.is_none() {
				e.try_insert(Slots::from(
					config
						.slots()
						.map(|(key, path)| (key, path.map(|path| server.load_asset(path)))),
				));
			}
		});
	}
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
	struct _Agent(_Config);

	impl GetFromSystemParam<()> for _Agent {
		type TParam<'w, 's> = ();
		type TItem<'i> = _Config;

		fn get_from_param<'a>(&self, _: &(), _: &()) -> Option<_Config> {
			Some(self.0.clone())
		}
	}

	#[derive(Debug, PartialEq, Clone)]
	struct _Config {
		inventory: Vec<Option<AssetPath<'static>>>,
		slots: Vec<(SlotKey, Option<AssetPath<'static>>)>,
	}

	impl LoadoutConfig for _Config {
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
		app.add_systems(Update, insert_internal::<_Agent, _Assets>);

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
		let entity = app
			.world_mut()
			.spawn(_Agent(_Config {
				inventory: vec![Some(AssetPath::from("inventory item"))],
				slots: vec![(SlotKey(42), Some(AssetPath::from("slot item")))],
			}))
			.id();

		app.update();

		let entity = app.world().entity(entity);
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
			_Agent(_Config {
				inventory: vec![Some(AssetPath::from("inventory item"))],
				slots: vec![(SlotKey(42), Some(AssetPath::from("slot item")))],
			}),
			Inventory::from([]),
		));

		app.update();
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
		let entity = app
			.world_mut()
			.spawn((
				_Agent(_Config {
					inventory: vec![Some(AssetPath::from("inventory item"))],
					slots: vec![(SlotKey(42), Some(AssetPath::from("slot item")))],
				}),
				Inventory::from([]),
			))
			.id();

		app.update();

		let entity = app.world().entity(entity);
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
			_Agent(_Config {
				inventory: vec![Some(AssetPath::from("inventory item"))],
				slots: vec![(SlotKey(42), Some(AssetPath::from("slot item")))],
			}),
			Slots::from([]),
		));

		app.update();
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
		let entity = app
			.world_mut()
			.spawn((
				_Agent(_Config {
					inventory: vec![Some(AssetPath::from("inventory item"))],
					slots: vec![(SlotKey(42), Some(AssetPath::from("slot item")))],
				}),
				Slots::from([]),
			))
			.id();

		app.update();

		let entity = app.world().entity(entity);
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
		let entity = app
			.world_mut()
			.spawn((
				_Agent(_Config {
					inventory: vec![Some(AssetPath::from("inventory item"))],
					slots: vec![(SlotKey(42), Some(AssetPath::from("slot item")))],
				}),
				Inventory::from([]),
				Slots::from([]),
			))
			.id();

		app.update();

		let entity = app.world().entity(entity);
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

	#[test]
	fn do_nothing_if_loadout_marker_present() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_load_asset::<Item, AssetPath<'static>>().never();
			mock.expect_load_asset::<Item, AssetPath<'static>>().never();
		}));
		app.world_mut().spawn((
			Loadout::<_Agent>::default(),
			_Agent(_Config {
				inventory: vec![Some(AssetPath::from("inventory item"))],
				slots: vec![(SlotKey(42), Some(AssetPath::from("slot item")))],
			}),
		));

		app.update();
	}
}
