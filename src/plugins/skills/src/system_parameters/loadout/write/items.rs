use crate::{
	components::{inventory::Inventory, slots::Slots},
	system_parameters::loadout::LoadoutWriter,
};
use bevy::prelude::*;
use common::{
	tools::inventory_key::InventoryKey,
	traits::{
		accessors::get::GetContextMut,
		handles_loadout::{
			LoadoutKey,
			items::{Items, SwapItems},
		},
	},
};

impl GetContextMut<Items> for LoadoutWriter<'_, '_> {
	type TContext<'ctx> = ItemsMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut LoadoutWriter,
		Items { entity }: Items,
	) -> Option<Self::TContext<'ctx>> {
		let (slots, inventory, ..) = param.agents.get_mut(entity).ok()?;

		Some(ItemsMut { inventory, slots })
	}
}

pub struct ItemsMut<'ctx> {
	inventory: Mut<'ctx, Inventory>,
	slots: Mut<'ctx, Slots>,
}

impl SwapItems for ItemsMut<'_> {
	fn swap_items<TA, TB>(&mut self, a: TA, b: TB)
	where
		TA: Into<LoadoutKey>,
		TB: Into<LoadoutKey>,
	{
		let a = a.into();
		let b = b.into();

		if a == b {
			return;
		}

		match (a, b) {
			(LoadoutKey::Inventory(InventoryKey(a)), LoadoutKey::Inventory(InventoryKey(b))) => {
				self.fill_inventory_up_to(a.max(b));

				let inventory = &mut self.inventory.0;

				inventory.swap(a, b);
			}
			(LoadoutKey::Slot(a), LoadoutKey::Slot(b)) => {
				let slots = &mut self.slots.items;
				let item_a = slots.remove(&a).flatten();
				let item_b = slots.remove(&b).flatten();

				slots.insert(a, item_b);
				slots.insert(b, item_a);
			}
			(LoadoutKey::Slot(slot_key), LoadoutKey::Inventory(InventoryKey(inventory_key)))
			| (LoadoutKey::Inventory(InventoryKey(inventory_key)), LoadoutKey::Slot(slot_key)) => {
				self.fill_inventory_up_to(inventory_key);

				let slot_item = self.slots.items.entry(slot_key).or_default();
				let Some(inventory_item) = self.inventory.0.get_mut(inventory_key) else {
					return;
				};

				std::mem::swap(slot_item, inventory_item);
			}
		}
	}
}

impl ItemsMut<'_> {
	fn fill_inventory_up_to(&mut self, index: usize) {
		if index < self.inventory.0.len() {
			return;
		}
		self.inventory.0.resize_with(index + 1, || None);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{combos::Combos, queue::Queue},
		skills::Skill,
	};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::action_key::slot::SlotKey;
	use std::ops::Deref;
	use testing::{IsChanged, SingleThreadedApp, new_handle};

	#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
	struct _ChangeDetection;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<Assets<Skill>>();
		app.add_systems(
			Update,
			(IsChanged::<Slots>::detect, IsChanged::<Inventory>::detect).in_set(_ChangeDetection),
		);

		app
	}

	mod get_context {
		use super::*;

		#[test]
		fn contains_components() -> Result<(), RunSystemError> {
			let slot_item = new_handle();
			let inventory_item = new_handle();
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Slots::from([(SlotKey(11), Some(slot_item.clone()))]),
					Inventory::from([None, Some(inventory_item.clone())]),
					Combos::default(),
					Queue::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |mut loadout: LoadoutWriter| {
					let ctx =
						LoadoutWriter::get_context_mut(&mut loadout, Items { entity }).unwrap();

					assert_eq!(
						(
							&Inventory::from([None, Some(inventory_item.clone())]),
							&Slots::from([(SlotKey(11), Some(slot_item.clone()))])
						),
						(ctx.inventory.deref(), ctx.slots.deref())
					);
				})
		}
	}

	mod swap {
		use super::*;
		use common::traits::thread_safe::ThreadSafe;
		use test_case::test_case;

		#[test]
		fn inventory_items() -> Result<(), RunSystemError> {
			let a = new_handle();
			let b = new_handle();
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Slots::default(),
					Inventory::from([Some(a.clone()), Some(b.clone())]),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |mut p: LoadoutWriter| {
					let mut ctx = LoadoutWriter::get_context_mut(&mut p, Items { entity }).unwrap();
					ctx.swap_items(InventoryKey(0), InventoryKey(1));
				})?;

			assert_eq!(
				Some(&Inventory::from([Some(b), Some(a)])),
				app.world().entity(entity).get::<Inventory>(),
			);
			Ok(())
		}

		#[test_case(InventoryKey(2), InventoryKey(0); "key a")]
		#[test_case(InventoryKey(0), InventoryKey(2); "key b")]
		fn inventory_items_just_out_of_bounds(
			key_a: InventoryKey,
			key_b: InventoryKey,
		) -> Result<(), RunSystemError> {
			let a = new_handle();
			let b = new_handle();
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Slots::default(),
					Inventory::from([Some(a.clone()), Some(b.clone())]),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |mut p: LoadoutWriter| {
					let mut ctx = LoadoutWriter::get_context_mut(&mut p, Items { entity }).unwrap();
					ctx.swap_items(key_a, key_b);
				})?;

			assert_eq!(
				Some(&Inventory::from([None, Some(b), Some(a)])),
				app.world().entity(entity).get::<Inventory>(),
			);
			Ok(())
		}

		#[test_case(InventoryKey(3), InventoryKey(0); "key a")]
		#[test_case(InventoryKey(0), InventoryKey(3); "key b")]
		fn inventory_items_out_of_bounds(
			key_a: InventoryKey,
			key_b: InventoryKey,
		) -> Result<(), RunSystemError> {
			let a = new_handle();
			let b = new_handle();
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Slots::default(),
					Inventory::from([Some(a.clone()), Some(b.clone())]),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |mut p: LoadoutWriter| {
					let mut ctx = LoadoutWriter::get_context_mut(&mut p, Items { entity }).unwrap();
					ctx.swap_items(key_a, key_b);
				})?;

			assert_eq!(
				Some(&Inventory::from([None, Some(b), None, Some(a)])),
				app.world().entity(entity).get::<Inventory>(),
			);
			Ok(())
		}

		#[test_case(InventoryKey(1), InventoryKey(0); "key a")]
		#[test_case(InventoryKey(0), InventoryKey(1); "key b")]
		fn inventory_not_shrunk(
			key_a: InventoryKey,
			key_b: InventoryKey,
		) -> Result<(), RunSystemError> {
			let a = new_handle();
			let b = new_handle();
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Slots::default(),
					Inventory::from([Some(a.clone()), Some(b.clone()), None, None]),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |mut p: LoadoutWriter| {
					let mut ctx = LoadoutWriter::get_context_mut(&mut p, Items { entity }).unwrap();
					ctx.swap_items(key_a, key_b);
				})?;

			assert_eq!(
				Some(&Inventory::from([Some(b), Some(a), None, None])),
				app.world().entity(entity).get::<Inventory>(),
			);
			Ok(())
		}

		#[test]
		fn slot_items() -> Result<(), RunSystemError> {
			let a = new_handle();
			let b = new_handle();
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Slots::from([
						(SlotKey(11), Some(a.clone())),
						(SlotKey(42), Some(b.clone())),
					]),
					Inventory::default(),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |mut p: LoadoutWriter| {
					let mut ctx = LoadoutWriter::get_context_mut(&mut p, Items { entity }).unwrap();
					ctx.swap_items(SlotKey(11), SlotKey(42));
				})?;

			assert_eq!(
				Some(&Slots::from([
					(SlotKey(11), Some(b.clone())),
					(SlotKey(42), Some(a.clone())),
				])),
				app.world().entity(entity).get::<Slots>(),
			);
			Ok(())
		}

		#[test_case(SlotKey(11), SlotKey(42); "in first key")]
		#[test_case(SlotKey(42), SlotKey(11); "in second key")]
		fn single_slot_item(key_a: SlotKey, key_b: SlotKey) -> Result<(), RunSystemError> {
			let item = new_handle();
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Slots::from([(SlotKey(11), Some(item.clone()))]),
					Inventory::default(),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |mut p: LoadoutWriter| {
					let mut ctx = LoadoutWriter::get_context_mut(&mut p, Items { entity }).unwrap();
					ctx.swap_items(key_a, key_b);
				})?;

			assert_eq!(
				Some(&Slots::from([
					(SlotKey(11), None),
					(SlotKey(42), Some(item.clone())),
				])),
				app.world().entity(entity).get::<Slots>(),
			);
			Ok(())
		}

		#[test_case(SlotKey(42), InventoryKey(1); "slots with inventory")]
		#[test_case(InventoryKey(1), SlotKey(42); "inventory with slots")]
		fn between_containers(
			key_a: impl Into<LoadoutKey> + Copy + ThreadSafe,
			key_b: impl Into<LoadoutKey> + Copy + ThreadSafe,
		) -> Result<(), RunSystemError> {
			let a = new_handle();
			let b = new_handle();
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Slots::from([(SlotKey(42), Some(a.clone()))]),
					Inventory::from([None, Some(b.clone())]),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |mut p: LoadoutWriter| {
					let mut ctx = LoadoutWriter::get_context_mut(&mut p, Items { entity }).unwrap();
					ctx.swap_items(key_a, key_b);
				})?;

			assert_eq!(
				(
					Some(&Inventory::from([None, Some(a)])),
					Some(&Slots::from([(SlotKey(42), Some(b))])),
				),
				(
					app.world().entity(entity).get::<Inventory>(),
					app.world().entity(entity).get::<Slots>(),
				),
			);
			Ok(())
		}

		#[test_case(SlotKey(42), InventoryKey(2); "slots with inventory")]
		#[test_case(InventoryKey(2), SlotKey(42); "inventory with slots")]
		fn between_containers_inventory_just_out_of_bounds(
			key_a: impl Into<LoadoutKey> + Copy + ThreadSafe,
			key_b: impl Into<LoadoutKey> + Copy + ThreadSafe,
		) -> Result<(), RunSystemError> {
			let item = new_handle();
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Slots::from([(SlotKey(42), Some(item.clone()))]),
					Inventory::from([None, None]),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |mut p: LoadoutWriter| {
					let mut ctx = LoadoutWriter::get_context_mut(&mut p, Items { entity }).unwrap();
					ctx.swap_items(key_a, key_b);
				})?;

			assert_eq!(
				(
					Some(&Inventory::from([None, None, Some(item)])),
					Some(&Slots::from([(SlotKey(42), None)])),
				),
				(
					app.world().entity(entity).get::<Inventory>(),
					app.world().entity(entity).get::<Slots>(),
				),
			);
			Ok(())
		}

		#[test_case(SlotKey(42), InventoryKey(3); "slots with inventory")]
		#[test_case(InventoryKey(3), SlotKey(42); "inventory with slots")]
		fn between_containers_inventory_out_of_bounds(
			key_a: impl Into<LoadoutKey> + Copy + ThreadSafe,
			key_b: impl Into<LoadoutKey> + Copy + ThreadSafe,
		) -> Result<(), RunSystemError> {
			let item = new_handle();
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Slots::from([(SlotKey(42), Some(item.clone()))]),
					Inventory::from([None, None]),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |mut p: LoadoutWriter| {
					let mut ctx = LoadoutWriter::get_context_mut(&mut p, Items { entity }).unwrap();
					ctx.swap_items(key_a, key_b);
				})?;

			assert_eq!(
				(
					Some(&Inventory::from([None, None, None, Some(item)])),
					Some(&Slots::from([(SlotKey(42), None)])),
				),
				(
					app.world().entity(entity).get::<Inventory>(),
					app.world().entity(entity).get::<Slots>(),
				),
			);
			Ok(())
		}

		#[test_case(SlotKey(42), InventoryKey(1); "slots with inventory")]
		#[test_case(InventoryKey(1), SlotKey(42); "inventory with slots")]
		fn between_containers_inventory_not_shrunk(
			key_a: impl Into<LoadoutKey> + Copy + ThreadSafe,
			key_b: impl Into<LoadoutKey> + Copy + ThreadSafe,
		) -> Result<(), RunSystemError> {
			let item = new_handle();
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Slots::from([(SlotKey(42), Some(item.clone()))]),
					Inventory::from([None, None, None, None]),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |mut p: LoadoutWriter| {
					let mut ctx = LoadoutWriter::get_context_mut(&mut p, Items { entity }).unwrap();
					ctx.swap_items(key_a, key_b);
				})?;

			assert_eq!(
				(
					Some(&Inventory::from([None, Some(item), None, None])),
					Some(&Slots::from([(SlotKey(42), None)])),
				),
				(
					app.world().entity(entity).get::<Inventory>(),
					app.world().entity(entity).get::<Slots>(),
				),
			);
			Ok(())
		}

		#[test_case(SlotKey(42), InventoryKey(1); "slots with inventory")]
		#[test_case(InventoryKey(1), SlotKey(42); "inventory with slots")]
		fn between_containers_with_empty_slot(
			key_a: impl Into<LoadoutKey> + Copy + ThreadSafe,
			key_b: impl Into<LoadoutKey> + Copy + ThreadSafe,
		) -> Result<(), RunSystemError> {
			let item = new_handle();
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Slots::default(),
					Inventory::from([None, Some(item.clone())]),
					Combos::default(),
				))
				.id();

			app.world_mut()
				.run_system_once(move |mut p: LoadoutWriter| {
					let mut ctx = LoadoutWriter::get_context_mut(&mut p, Items { entity }).unwrap();
					ctx.swap_items(key_a, key_b);
				})?;

			assert_eq!(
				(
					Some(&Inventory::from([None, None])),
					Some(&Slots::from([(SlotKey(42), Some(item))])),
				),
				(
					app.world().entity(entity).get::<Inventory>(),
					app.world().entity(entity).get::<Slots>(),
				),
			);
			Ok(())
		}

		#[test_case(InventoryKey(2), InventoryKey(2); "inventory key")]
		#[test_case(SlotKey(11), SlotKey(11); "slot key")]
		fn components_not_changed_when_using_same(
			key_a: impl Into<LoadoutKey> + Copy + ThreadSafe,
			key_b: impl Into<LoadoutKey> + Copy + ThreadSafe,
		) {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					Slots::from([(SlotKey(11), Some(new_handle()))]),
					Inventory::from([None, None, Some(new_handle()), None]),
					Combos::default(),
				))
				.id();
			app.add_systems(
				Update,
				(move |mut p: LoadoutWriter| {
					let mut ctx = LoadoutWriter::get_context_mut(&mut p, Items { entity }).unwrap();
					ctx.swap_items(key_a, key_b);
				})
				.before(_ChangeDetection),
			);

			app.update();
			app.update();

			assert_eq!(
				(Some(&IsChanged::FALSE), Some(&IsChanged::FALSE)),
				(
					app.world().entity(entity).get::<IsChanged<Slots>>(),
					app.world().entity(entity).get::<IsChanged<Inventory>>(),
				)
			);
		}
	}
}
