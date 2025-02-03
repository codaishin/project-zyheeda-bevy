use bevy::prelude::*;
use common::{
	components::{Collection, Swap},
	tools::inventory_key::InventoryKey,
	traits::try_remove_from::TryRemoveFrom,
};
use skills::{components::inventory::Inventory, item::Item};
use std::cmp::max;

type ItemsToSwap<'a> = (
	Entity,
	&'a mut Inventory,
	&'a Collection<Swap<InventoryKey, InventoryKey>>,
);

pub fn swap_inventory_items(mut commands: Commands, mut items_to_swap: Query<ItemsToSwap>) {
	for (agent, mut inventory, swaps) in &mut items_to_swap {
		for swap in &swaps.0 {
			do_swap(&mut inventory, swap);
		}

		commands.try_remove_from::<Collection<Swap<InventoryKey, InventoryKey>>>(agent);
	}
}

fn do_swap(inventory: &mut Mut<Inventory>, swap: &Swap<InventoryKey, InventoryKey>) {
	fill_to(&mut inventory.0, max(swap.0 .0, swap.1 .0));
	inventory.0.swap(swap.0 .0, swap.1 .0);
}

fn fill_to(inventory: &mut Vec<Option<Handle<Item>>>, index: usize) {
	if index < inventory.len() {
		return;
	}

	inventory.extend(vec![None; index - inventory.len() + 1]);
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::test_tools::utils::new_handle;

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn swap_items() -> Result<(), RunSystemError> {
		let item_a = new_handle();
		let item_b = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Inventory::new([Some(item_a.clone()), None, Some(item_b.clone())]),
				Collection::new([Swap(InventoryKey(0), InventoryKey(2))]),
			))
			.id();

		app.world_mut().run_system_once(swap_inventory_items)?;

		assert_eq!(
			Some(&Inventory::new([Some(item_b), None, Some(item_a)])),
			app.world().entity(agent).get::<Inventory>()
		);
		Ok(())
	}

	#[test]
	fn swap_items_out_or_range() -> Result<(), RunSystemError> {
		let item = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Inventory::new([Some(item.clone())]),
				Collection::new([Swap(InventoryKey(0), InventoryKey(2))]),
			))
			.id();

		app.world_mut().run_system_once(swap_inventory_items)?;

		assert_eq!(
			Some(&Inventory::new([None, None, Some(item)])),
			app.world().entity(agent).get::<Inventory>()
		);
		Ok(())
	}

	#[test]
	fn swap_items_index_and_len_are_same() -> Result<(), RunSystemError> {
		let item = new_handle();
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Inventory::new([Some(item.clone())]),
				Collection::new([Swap(InventoryKey(0), InventoryKey(1))]),
			))
			.id();

		app.world_mut().run_system_once(swap_inventory_items)?;

		assert_eq!(
			Some(&Inventory::new([None, Some(item)])),
			app.world().entity(agent).get::<Inventory>()
		);
		Ok(())
	}

	#[test]
	fn remove_swap_collection() -> Result<(), RunSystemError> {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				Inventory::new([]),
				Collection::<Swap<InventoryKey, InventoryKey>>::new([]),
			))
			.id();

		app.world_mut().run_system_once(swap_inventory_items)?;

		let agent = app.world().entity(agent);

		assert!(!agent.contains::<Collection<Swap<InventoryKey, InventoryKey>>>());
		Ok(())
	}
}
