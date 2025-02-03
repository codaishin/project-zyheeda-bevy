use bevy::prelude::*;
use common::{
	components::{Collection, Swap},
	tools::inventory_key::InventoryKey,
	traits::{handles_equipment::ContinuousAccessMut, try_remove_from::TryRemoveFrom},
};
use std::cmp::max;

impl<T> SwapInventoryItems for T {}

pub trait SwapInventoryItems {
	fn swap_items(mut commands: Commands, mut items_to_swap: Query<ItemsToSwap<Self>>)
	where
		Self: Component + ContinuousAccessMut + Sized,
		Self::TItem: Clone,
	{
		for (agent, mut inventory, swaps) in &mut items_to_swap {
			for swap in &swaps.0 {
				do_swap(&mut inventory, swap);
			}

			commands.try_remove_from::<Collection<Swap<InventoryKey, InventoryKey>>>(agent);
		}
	}
}

type ItemsToSwap<'a, TInventory> = (
	Entity,
	&'a mut TInventory,
	&'a Collection<Swap<InventoryKey, InventoryKey>>,
);

fn do_swap<TInventory>(inventory: &mut Mut<TInventory>, swap: &Swap<InventoryKey, InventoryKey>)
where
	TInventory: Component + ContinuousAccessMut,
	TInventory::TItem: Clone,
{
	let items = inventory.continuous_access_mut();
	fill_to(items, max(swap.0 .0, swap.1 .0));
	items.swap(swap.0 .0, swap.1 .0);
}

fn fill_to<TItem>(items: &mut Vec<Option<TItem>>, index: usize)
where
	TItem: Clone,
{
	if index < items.len() {
		return;
	}

	items.extend(vec![None; index - items.len() + 1]);
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};

	#[derive(Debug, PartialEq, Clone, Copy)]
	struct _Item(&'static str);

	#[derive(Component, Debug, PartialEq)]
	struct _Inventory(Vec<Option<_Item>>);

	impl ContinuousAccessMut for _Inventory {
		type TItem = _Item;

		fn continuous_access_mut(&mut self) -> &mut Vec<Option<Self::TItem>> {
			&mut self.0
		}
	}

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn swap_items() -> Result<(), RunSystemError> {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Inventory(vec![Some(_Item("a")), None, Some(_Item("b"))]),
				Collection::new([Swap(InventoryKey(0), InventoryKey(2))]),
			))
			.id();

		app.world_mut().run_system_once(_Inventory::swap_items)?;

		assert_eq!(
			Some(&_Inventory(vec![Some(_Item("b")), None, Some(_Item("a"))])),
			app.world().entity(agent).get::<_Inventory>()
		);
		Ok(())
	}

	#[test]
	fn swap_items_out_or_range() -> Result<(), RunSystemError> {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Inventory(vec![Some(_Item("a"))]),
				Collection::new([Swap(InventoryKey(0), InventoryKey(2))]),
			))
			.id();

		app.world_mut().run_system_once(_Inventory::swap_items)?;

		assert_eq!(
			Some(&_Inventory(vec![None, None, Some(_Item("a"))])),
			app.world().entity(agent).get::<_Inventory>()
		);
		Ok(())
	}

	#[test]
	fn swap_items_index_and_len_are_same() -> Result<(), RunSystemError> {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Inventory(vec![Some(_Item("a"))]),
				Collection::new([Swap(InventoryKey(0), InventoryKey(1))]),
			))
			.id();

		app.world_mut().run_system_once(_Inventory::swap_items)?;

		assert_eq!(
			Some(&_Inventory(vec![None, Some(_Item("a"))])),
			app.world().entity(agent).get::<_Inventory>()
		);
		Ok(())
	}

	#[test]
	fn remove_swap_collection() -> Result<(), RunSystemError> {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Inventory(vec![]),
				Collection::<Swap<InventoryKey, InventoryKey>>::new([]),
			))
			.id();

		app.world_mut().run_system_once(_Inventory::swap_items)?;

		let agent = app.world().entity(agent);

		assert!(!agent.contains::<Collection<Swap<InventoryKey, InventoryKey>>>());
		Ok(())
	}
}
