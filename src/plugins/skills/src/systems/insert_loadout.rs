use crate::components::{inventory::Inventory, loadout::Loadout, slots::Slots};
use bevy::{asset::AssetPath, prelude::*};
use common::{
	traits::{
		accessors::get::TryApplyOn,
		load_asset::LoadAsset,
		loadout::{ItemName, LoadoutConfig},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<TAgent> Loadout<TAgent>
where
	TAgent: Component + LoadoutConfig,
{
	#[allow(clippy::type_complexity)]
	pub(crate) fn insert(
		agents: Query<
			(Entity, &TAgent, Option<&Inventory>, Option<&Slots>),
			Without<Loadout<TAgent>>,
		>,
		commands: ZyheedaCommands,
		server: ResMut<AssetServer>,
	) {
		insert_internal(agents, commands, server);
	}
}

#[allow(clippy::type_complexity)]
fn insert_internal<TAgent, TAssetServer>(
	agents: Query<(Entity, &TAgent, Option<&Inventory>, Option<&Slots>), Without<Loadout<TAgent>>>,
	mut commands: ZyheedaCommands,
	mut server: ResMut<TAssetServer>,
) where
	TAgent: Component + LoadoutConfig,
	TAssetServer: Resource + LoadAsset,
{
	for (entity, agent, inventory, slots) in &agents {
		commands.try_apply_on(&entity, |mut e| {
			e.try_insert(Loadout::<TAgent>::default());

			if inventory.is_none() {
				e.try_insert(Inventory::from(
					agent
						.inventory()
						.map(|name| name.map(asset_path))
						.map(|path| path.map(|path| server.load_asset(path))),
				));
			}

			if slots.is_none() {
				e.try_insert(Slots::from(
					agent
						.slots()
						.map(|(key, name)| (key, name.map(asset_path)))
						.map(|(key, path)| (key, path.map(|p| server.load_asset(p)))),
				));
			}
		});
	}
}

fn asset_path(ItemName(name): ItemName) -> AssetPath<'static> {
	AssetPath::from(format!("items/{name}.item"))
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{tools::action_key::slot::SlotKey, traits::load_asset::mock::MockAssetServer};
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Agent {
		inventory: Vec<Option<ItemName>>,
		slots: Vec<(SlotKey, Option<ItemName>)>,
	}

	impl LoadoutConfig for _Agent {
		fn inventory(&self) -> impl Iterator<Item = Option<ItemName>> {
			self.inventory.iter().cloned()
		}

		fn slots(&self) -> impl Iterator<Item = (SlotKey, Option<ItemName>)> {
			self.slots.iter().cloned()
		}
	}

	fn setup(assets: MockAssetServer) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(assets);
		app.add_systems(Update, insert_internal::<_Agent, MockAssetServer>);

		app
	}

	#[test]
	fn insert_loadout_components() {
		let inventory_item = new_handle();
		let slot_item = new_handle();
		let mut app = setup(
			MockAssetServer::default()
				.path("items/inventory item.item")
				.returns(inventory_item.clone())
				.path("items/slot item.item")
				.returns(slot_item.clone()),
		);
		let entity = app
			.world_mut()
			.spawn(_Agent {
				inventory: vec![Some(ItemName::from("inventory item"))],
				slots: vec![(SlotKey(42), Some(ItemName::from("slot item")))],
			})
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
		let mut app = setup(MockAssetServer::default());
		app.world_mut().spawn((
			_Agent {
				inventory: vec![Some(ItemName::from("inventory item"))],
				slots: vec![(SlotKey(42), Some(ItemName::from("slot item")))],
			},
			Inventory::from([]),
		));

		app.update();

		assert_eq!(
			0,
			app.world()
				.resource::<MockAssetServer>()
				.calls("items/inventory item.item"),
		);
	}

	#[test]
	fn insert_loadout_if_inventory_already_present() {
		let slot_item = new_handle();
		let mut app = setup(
			MockAssetServer::default()
				.path("items/slot item.item")
				.returns(slot_item.clone()),
		);
		let entity = app
			.world_mut()
			.spawn((
				_Agent {
					inventory: vec![Some(ItemName::from("inventory item"))],
					slots: vec![(SlotKey(42), Some(ItemName::from("slot item")))],
				},
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
		let mut app = setup(MockAssetServer::default());
		app.world_mut().spawn((
			_Agent {
				inventory: vec![Some(ItemName::from("inventory item"))],
				slots: vec![(SlotKey(42), Some(ItemName::from("slot item")))],
			},
			Slots::from([]),
		));

		app.update();

		assert_eq!(
			0,
			app.world()
				.resource::<MockAssetServer>()
				.calls("items/slot item.item"),
		);
	}

	#[test]
	fn insert_loadout_if_slots_already_present() {
		let inventory_item = new_handle();
		let mut app = setup(
			MockAssetServer::default()
				.path("items/inventory item.item")
				.returns(inventory_item.clone()),
		);
		let entity = app
			.world_mut()
			.spawn((
				_Agent {
					inventory: vec![Some(ItemName::from("inventory item"))],
					slots: vec![(SlotKey(42), Some(ItemName::from("slot item")))],
				},
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
		let mut app = setup(MockAssetServer::default());
		let entity = app
			.world_mut()
			.spawn((
				_Agent {
					inventory: vec![Some(ItemName::from("inventory item"))],
					slots: vec![(SlotKey(42), Some(ItemName::from("slot item")))],
				},
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
		let mut app = setup(MockAssetServer::default());
		app.world_mut().spawn((
			Loadout::<_Agent>::default(),
			_Agent {
				inventory: vec![Some(ItemName::from("inventory item"))],
				slots: vec![(SlotKey(42), Some(ItemName::from("slot item")))],
			},
		));

		app.update();

		let server = app.world().resource::<MockAssetServer>();
		assert_eq!(
			(0, 0),
			(
				server.calls("items/inventory item.item"),
				server.calls("items/slot item.item"),
			),
		);
	}
}
